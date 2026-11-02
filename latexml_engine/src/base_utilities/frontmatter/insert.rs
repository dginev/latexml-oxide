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
  link_entry_addresses(document)?;
  respell_emails(document)?;
  show_unanswered_marks(document, &answered)?;
  drop_empty_contacts(document)?;
  // (FM S2b: a written family's paper, where it differs from the observer's reading by the writer's operations alone,
  // written as the reading has it — under the shadow switch only; OXIDIZED_DESIGN_DIVERGENCES #837 "S2b")
  write_observed_frontmatter(document)?;
  // (the authors as written: a name the writer re-parted holds one person, OD #928)
  flag_merged_creators(document)?;
  // (the unclaimed-text net's notes: their lines no frontmatter field and no text already set holds)
  keep_unclaimed_net_notes(document)?;
  Ok(())
}

/// The words a note line counts for [`keep_unclaimed_net_notes`] (the frontmatter scorer's, fm_scorer/score_tiers.py
/// `content_words`): its letter runs of two letters or more outside the stop words and its digit runs of three or more,
/// folded — accents dropped ([`resolver::base_letters`]), lower-cased.
fn net_content_words(text: &str) -> Vec<String> {
  const STOPWORDS: &[&str] = &[
    "a", "an", "the", "of", "and", "or", "in", "on", "at", "for", "to", "from", "by", "with",
    "via", "de", "la", "le", "les", "du", "des", "del", "della", "di", "da", "do", "dos", "das",
    "e", "y", "et", "und", "von", "van", "der", "den", "zu", "im", "am", "al", "el", "en", "is",
    "are", "be", "as", "this", "that", "its", "it", "we", "our", "their", "his", "her", "no",
    "not",
  ];
  let folded = resolver::base_letters(text).to_lowercase();
  let mut words = Vec::new();
  let mut run = String::new();
  let mut letters = false;
  let flush = |run: &mut String, letters: bool, words: &mut Vec<String>| {
    let keep = if letters {
      run.len() >= 2 && !STOPWORDS.contains(&run.as_str())
    } else {
      run.len() >= 3
    };
    if keep {
      words.push(std::mem::take(run));
    } else {
      run.clear();
    }
  };
  for c in folded.chars() {
    let kind = if c.is_ascii_lowercase() {
      Some(true)
    } else if c.is_ascii_digit() {
      Some(false)
    } else {
      None
    };
    if kind != Some(letters) && !run.is_empty() {
      flush(&mut run, letters, &mut words);
    }
    if let Some(kind) = kind {
      letters = kind;
      run.push(c);
    }
  }
  if !run.is_empty() {
    flush(&mut run, letters, &mut words);
  }
  words
}

/// The text of `node` without what the nodes `skip` hold.
fn text_outside(node: &Node, skip: &[Node], out: &mut String) {
  for child in node.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) {
      if !skip.contains(&child) {
        text_outside(&child, skip, out);
        out.push(' ');
      }
    } else {
      out.push_str(&child.get_content());
    }
  }
}

/// A note's lines ([`keep_unclaimed_net_notes`]): what stands between its `ltx:break`s, each `ltx:p` apart, with the
/// break that ends it.
fn net_note_lines(parent: &Node, lines: &mut Vec<(Vec<Node>, String)>) {
  let mut line: (Vec<Node>, String) = (Vec::new(), String::new());
  for child in parent.get_child_nodes() {
    let element = child.get_type() == Some(NodeType::ElementNode);
    if element && child.get_name() == "break" {
      line.0.push(child);
      lines.push(std::mem::take(&mut line));
    } else if element && child.get_name() == "p" {
      if !line.0.is_empty() {
        lines.push(std::mem::take(&mut line));
      }
      net_note_lines(&child, lines);
    } else {
      line.1.push_str(&child.get_content());
      line.1.push(' ');
      line.0.push(child);
    }
  }
  if !line.0.is_empty() {
    lines.push(line);
  }
}

/// Remove the `ltx:break`s a note starts or ends with, blank text aside (the lines [`keep_unclaimed_net_notes`] took out
/// leave them: `\\AB@affillist` starts with its `\\\\`).
fn trim_edge_breaks(document: &mut Document, note: &Node) {
  let blank =
    |n: &Node| n.get_type() != Some(NodeType::ElementNode) && n.get_content().trim().is_empty();
  for from_end in [false, true] {
    loop {
      let mut children = note.get_child_nodes();
      if from_end {
        children.reverse();
      }
      match children.into_iter().find(|n| !blank(n)) {
        Some(edge)
          if edge.get_type() == Some(NodeType::ElementNode) && edge.get_name() == "break" =>
        {
          document.remove_node(edge);
        },
        _ => break,
      }
    }
  }
}

/// The unclaimed-text net's notes (frontmatter_stores.rs `hand_net_stores`: what a raw class's title page prints of a
/// store no frontmatter owner claims, OXIDIZED_DESIGN_DIVERGENCES #953), once the frontmatter is placed: a note line
/// (split at `ltx:break` and `ltx:p`, [`net_note_lines`]) is kept whole only when it holds 3 or more distinct content
/// words ([`net_content_words`]) that neither the frontmatter nor the text set before it holds — the whole document's
/// text but the net's notes: the deposit's paragraphs, the creators (`\AB@names` repeats the names), the abstract —
/// else it goes; a note left without a line goes, with a nameless creator it leaves empty. Each kept note is reported
/// (FIDELITY_CHARTER §4: printed text no owner claims, kept beside the frontmatter, counted).
fn keep_unclaimed_net_notes(document: &mut Document) -> Result<()> {
  let notes = document.findnodes("//ltx:note[@_net]", None);
  if notes.is_empty() {
    return Ok(());
  }
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(());
  };
  let mut claimed_text = String::new();
  text_outside(&root, &notes, &mut claimed_text);
  let claimed: HashSet<String> = net_content_words(&claimed_text).into_iter().collect();
  for mut note in notes {
    let store = note.get_attribute("_net").unwrap_or_default();
    // (and the words of the class's own code in the store, frontmatter_stores.rs `net_class_text`)
    let class: HashSet<String> =
      net_content_words(&lookup_string(&s!("lx_net_class_text_{store}")))
        .into_iter()
        .collect();
    let mut lines = Vec::new();
    net_note_lines(&note, &mut lines);
    let mut kept = 0usize;
    for (nodes, text) in lines {
      let mut unmatched: Vec<String> = net_content_words(&text)
        .into_iter()
        .filter(|word| !claimed.contains(word) && !class.contains(word))
        .collect();
      unmatched.sort();
      unmatched.dedup();
      if unmatched.len() >= 3 {
        kept += 1;
        continue;
      }
      for node in nodes {
        document.remove_node(node);
      }
    }
    // (the breaks left at either end of a kept line's run, and the paragraphs left empty)
    for p in document.findnodes(".//ltx:p", Some(&note)) {
      if element_nodes(&p).is_empty() && p.get_content().trim().is_empty() {
        document.remove_node(p);
      }
    }
    let creator = note.get_parent();
    if kept == 0 {
      document.remove_node(note);
      if let Some(creator) = creator
        && creator.get_name() == "creator"
        && element_nodes(&creator).is_empty()
      {
        document.remove_node(creator);
      }
      continue;
    }
    trim_edge_breaks(document, &note);
    note.remove_attribute("_net").ok();
    Warn!(
      "frontmatter",
      "unclaimed",
      s!(
        "The title page prints {kept} line(s) of \\{store} that no frontmatter field holds: kept as a note of the author block"
      )
    );
  }
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

/// The Link step for the addresses printed inside an affiliation-kind entry (OXIDIZED_DESIGN_DIVERGENCES #956): every
/// e-mail digested inside one (an `\email` in an `\institute` entry, an address line the Kind step made one,
/// `affiliations::kind_entry_addresses`), wherever its label or call placed it, goes where FIDELITY_CHARTER §4's chain
/// over that entry puts it ([`entry_address_owners`], the frontmatter observer's chain too): the author it spells, else
/// the entry's only carrier, else every carrier of a symbol-led entry, else one per carrier in order, else the block's
/// own creator, once (ruling E, main session 2026-11-02). The entry's carriers are the authors holding it after
/// relocation (an entry of addresses alone stays as their carrier, empty, until `drop_empty_contacts`), else those
/// holding its e-mails. Each placed e-mail follows the entry on its owner (user ruling R8, print order). An entry whose
/// e-mails already stand as the chain puts them is left as it is. Witnesses: 2502.01754, 2602.21641, 2307.02047,
/// 1901.09107, 0902.3918, 1909.00276, astro-ph0512555; guards `perfect_kernel_batch100b`.
fn link_entry_addresses(document: &mut Document) -> Result<()> {
  let emails = document.findnodes(
    &s!("//ltx:creator/ltx:contact[@role='email'][@{ENTRY_KEY}]"),
    None,
  );
  if emails.is_empty() {
    return Ok(());
  }
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  let names: Vec<String> = authors
    .iter()
    .map(|author| {
      document
        .findnode("ltx:personname", Some(author))
        .map(|person| person.get_content())
        .unwrap_or_default()
    })
    .collect();
  // (each name read by its base letters, as the frontmatter observer's chain reads it: `joze.rozanec@` spells Jože M.
  // Rožanec, 2105.01929 — `resolver::base_letters`)
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(&resolver::base_letters(name)))))
    .collect();
  let position = |creator: &Node| authors.iter().position(|a| a == creator);
  // (each entry's holders, with their copies of it; the entries of one printed text are one: the same affiliation set
  // apart for each author, aastex's `\affil` after each name, its address no one carrier's by being printed under each —
  // as the frontmatter observer's reading groups its prints, 2212.12876)
  // (an entry's printed text, its keys, its holders with their copies)
  type EntryGroup = (String, Vec<String>, Vec<(usize, Node)>);
  let mut groups: Vec<EntryGroup> = Vec::new();
  for email in &emails {
    let key = email.get_attribute(ENTRY_KEY).unwrap_or_default();
    if groups
      .iter()
      .any(|(_, entry_keys, _)| entry_keys.contains(&key))
    {
      continue;
    }
    let held: Vec<(usize, Node)> = document
      .findnodes(
        &s!("//ltx:creator/ltx:contact[@{ENTRY_KEY}='{key}'][@role!='email']"),
        None,
      )
      .into_iter()
      .filter_map(|entry| {
        let at = position(&entry.get_parent()?)?;
        Some((at, entry))
      })
      .collect();
    // (an entry of addresses alone is its addresses: two such calls printing the same list, REVTeX's
    // `\altaffiliation[Authors for Correspondence: ]{Email: nikhil@…, bala@…}` after each of two names, are one —
    // 2208.10784)
    let text = held
      .first()
      .map(|(_, entry)| {
        entry
          .get_content()
          .split_whitespace()
          .collect::<Vec<_>>()
          .join(" ")
      })
      .filter(|text| !text.is_empty())
      .unwrap_or_else(|| {
        let mut own: Vec<&Node> = emails
          .iter()
          .filter(|e| e.get_attribute(ENTRY_KEY).as_deref() == Some(key.as_str()))
          .collect();
        own.sort_by_key(|e| {
          e.get_attribute(QUEUE_ORDINAL)
            .and_then(|n| n.parse::<i64>().ok())
            .unwrap_or(i64::MAX)
        });
        let addresses: Vec<String> = own
          .iter()
          .map(|e| e.get_content().trim().to_string())
          .collect();
        s!("\u{1}{}", addresses.join("\u{2}"))
      });
    match groups.iter_mut().find(|(t, ..)| *t == text) {
      Some((_, entry_keys, all)) => {
        entry_keys.push(key);
        all.extend(held);
      },
      None => groups.push((text, vec![key], held)),
    }
  }
  // (the addresses each entry leaves to the block, given once after every entry is placed, and only where no author
  // holds them: an address is never duplicated — 2208.10784's `bala@` on its author by one copy of a shared entry)
  let mut to_block: Vec<(String, Node)> = Vec::new();
  let mut taken_away: Vec<Node> = Vec::new();
  for (entry_text, entry_keys, held) in groups {
    // (in print order: the order their calls were digested in, `QUEUE_ORDINAL`)
    let mut mine: Vec<Node> = emails
      .iter()
      .filter(|e| {
        e.get_attribute(ENTRY_KEY)
          .is_some_and(|k| entry_keys.contains(&k))
      })
      .cloned()
      .collect();
    mine.sort_by_key(|e| {
      e.get_attribute(QUEUE_ORDINAL)
        .and_then(|n| n.parse::<i64>().ok())
        .unwrap_or(i64::MAX)
    });
    let key = entry_keys.join(",");
    let mut carriers: Vec<usize> = held.iter().map(|(at, _)| *at).collect();
    if carriers.is_empty() {
      carriers = mine
        .iter()
        .filter_map(|e| position(&e.get_parent()?))
        .collect();
    }
    carriers.sort_unstable();
    carriers.dedup();
    // (its addresses, one each, in print order; each with the authors holding it now — an e-mail printing several,
    // `\email{a|b@x.org}` inside a line of the entry, written out, each its own, as the frontmatter observer reads it:
    // `resolver::addresses_in`)
    // (an e-mail printing addresses alone, several written out; one printing a template, a list the reader could not
    // part or a group beside words — `{ann,bob}@a.org (corresponding)`, 92c — is no one's address: it stays where its
    // call placed it, as the frontmatter observer's reading leaves it)
    let readable = |content: &str| {
      let listed = resolver::addresses_in(content);
      let (_, rest) = resolver::address_take_out(content, &[]);
      // (an e-mail label beside a group keeps it whole, as the frontmatter observer's group write-out does:
      // `Emails: {ann,bob}@a.org`, 92c)
      let words: Vec<&str> = content
        .split(|c: char| c.is_whitespace() || ",;:".contains(c))
        .filter(|w| !w.is_empty())
        .collect();
      // (a label of one word or two, "Electronic Addresses", 1509.07506)
      let labelled = words.iter().any(|w| is_email_label(w))
        || words.windows(2).any(|pair| is_email_label(&pair.join(" ")));
      if listed.len() > 1 && !labelled {
        (!rest.chars().any(char::is_alphanumeric)).then_some(listed)
      } else if listed.len() > 1 {
        None
      } else {
        let text = content.trim();
        (!lists_local_parts(text) && !is_pattern_address(text)).then(|| vec![text.to_string()])
      }
    };
    // (a contact printing several addresses and more — an ORCID's link, math, a note — is never written out: it stays
    // whole where its call placed it, [`prints_text_only`]; 2308.02269, review rev100b B1)
    mine.retain(|email| {
      readable(&email.get_content())
        .is_some_and(|listed| listed.len() < 2 || prints_text_only(email))
    });
    if mine.is_empty() {
      continue;
    }
    let mut addresses: Vec<(String, Node, Vec<Option<usize>>)> = Vec::new();
    let mut written: Vec<(Node, String)> = Vec::new();
    for email in &mine {
      let content = email.get_content();
      let listed = readable(&content).unwrap_or_default();
      let texts: Vec<String> = if listed.len() > 1 {
        written.extend(
          listed
            .iter()
            .map(|address| (email.clone(), address.clone())),
        );
        listed
      } else {
        vec![content.trim().to_string()]
      };
      let holder = email.get_parent().and_then(|p| position(&p));
      for text in texts {
        match addresses.iter_mut().find(|(t, ..)| *t == text) {
          Some((.., holders)) => holders.push(holder),
          None => addresses.push((text, email.clone(), vec![holder])),
        }
      }
    }
    let symbol = mine.iter().any(|e| e.get_attribute(ENTRY_SYMBOL).is_some());
    let spelled: Vec<Option<usize>> = addresses
      .iter()
      .map(|(text, ..)| {
        text
          .split_whitespace()
          .find(|word| word.contains('@'))
          .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
          .filter(|address| !is_entry_word_mailbox(address, &entry_text))
          .and_then(|address| spelled_owner(&keys, address))
      })
      .collect();
    // (an e-mail command the author wrote inside the entry, holding one address, keeps the one author its own call
    // placed it on, unless it spells another: FIDELITY_CHARTER §4 #420, an author's own call holding one address keeps
    // it — jmlr's `\Name{…} \Email{costis@…}` under a misread `\AND`, 2002.00057, 1504.01446, 2509.23923; not an
    // address the Kind step took out of the entry's text, `ENTRY_KINDED`)
    let own_call: Vec<Option<usize>> = addresses
      .iter()
      .zip(&spelled)
      .map(|((text, _, holders), spelled)| {
        let written_by_author = mine.iter().any(|e| {
          e.get_attribute(ENTRY_KINDED).is_none() && e.get_content().trim() == text.as_str()
        });
        match holders.as_slice() {
          [Some(holder)]
            if written_by_author
              && !written.iter().any(|(_, a)| a == text)
              && spelled.is_none_or(|s| s == *holder) =>
          {
            Some(*holder)
          },
          _ => None,
        }
      })
      .collect();
    // (the carriers holding an address of their own, outside this entry)
    let addressed: Vec<usize> = carriers
      .iter()
      .copied()
      .filter(|&c| {
        element_nodes(&authors[c]).iter().any(|n| {
          n.get_name() == "contact"
            && n.get_attribute("role").as_deref() == Some("email")
            && !mine.contains(n)
        })
      })
      .collect();
    let owners: Vec<(Option<Vec<usize>>, &'static str)> = entry_address_owners(
      &spelled,
      0..addresses.len(),
      &carriers,
      symbol,
      &addressed,
      authors.len() == 1,
    )
    .into_iter()
    .zip(&own_call)
    .map(|(owner, own)| match own {
      Some(holder) => (Some(vec![*holder]), "owner:call"),
      None => owner,
    })
    .collect();
    // (already as the chain puts them: left as they are)
    let settled = written.is_empty()
      && addresses
        .iter()
        .zip(&owners)
        .all(|((.., holders), (to, _))| {
          let mut now: Vec<Option<usize>> = holders.clone();
          now.sort_unstable();
          let want: Vec<Option<usize>> = match to {
            Some(to) => {
              let mut to: Vec<Option<usize>> = to.iter().map(|&a| Some(a)).collect();
              to.sort_unstable();
              to
            },
            None => vec![None],
          };
          now == want
        });
    if settled {
      // (each after the entry on its holder, in print order: an inner e-mail an `annotate=` placed is attached before
      // the entry it was digested in)
      for (at, entry) in &held {
        let mut anchor = entry.clone();
        for email in &mine {
          if email.get_parent().as_ref() == Some(&authors[*at]) {
            let mut email = email.clone();
            if anchor.add_next_sibling(&mut email).is_ok() {
              anchor = email;
            }
          }
        }
      }
      continue;
    }
    DebugFeature!(
      "frontmatter",
      "FRONT link entry {key}: carriers {carriers:?}, owners {:?}",
      owners
    );
    let emptied: Vec<Node> = mine.iter().filter_map(Node::get_parent).collect();
    // (the copies made while the entry's e-mails still stand, the templates; those taken away after)
    let mut after: HashMap<usize, Node> = HashMap::default();
    for ((text, template, _), (to, _)) in addresses.iter().zip(owners) {
      match to {
        Some(to) => {
          for owner in to {
            let mut author = authors[owner].clone();
            if element_nodes(&author).iter().any(|c| {
              !mine.contains(c)
                && c.get_name() == "contact"
                && c.get_attribute("role").as_deref() == Some("email")
                && c.get_content().trim() == text.as_str()
            }) {
              continue;
            }
            document.append_clone(&mut author, vec![template.clone()])?;
            let Some(mut placed) = element_nodes(&author).last().cloned() else {
              continue;
            };
            if written
              .iter()
              .any(|(node, address)| node == template && *address == *text)
            {
              set_written_out(&placed, text);
            }
            // (after the entry, or the address placed after it before)
            let anchor = after.get(&owner).cloned().or_else(|| {
              held
                .iter()
                .filter(|(at, _)| *at == owner)
                .map(|(_, entry)| entry.clone())
                .next_back()
            });
            if let Some(mut anchor) = anchor
              && anchor.add_next_sibling(&mut placed).is_err()
            {
              continue;
            }
            after.insert(owner, placed);
          }
        },
        None => {
          let Some(mut block) = block_creator(document, &authors)? else {
            continue;
          };
          document.append_clone(&mut block, vec![template.clone()])?;
          if let Some(placed) = element_nodes(&block).last().cloned() {
            if written
              .iter()
              .any(|(node, address)| node == template && *address == *text)
            {
              set_written_out(&placed, text);
            }
            to_block.push((text.clone(), placed));
          }
        },
      }
    }
    for email in &mine {
      document.remove_node(email.clone());
    }
    taken_away.extend(emptied);
  }
  // (a block copy an author holds, or a second one on the block, goes)
  let mut on_block: Vec<String> = Vec::new();
  for (text, placed) in to_block {
    let held = authors.iter().any(|author| {
      element_nodes(author).iter().any(|c| {
        c.get_name() == "contact"
          && c.get_attribute("role").as_deref() == Some("email")
          && c.get_content().trim() == text
      })
    });
    if held || on_block.contains(&text) {
      if let Some(parent) = placed.get_parent() {
        taken_away.push(parent);
      }
      document.remove_node(placed);
    } else {
      on_block.push(text);
    }
  }
  // (a nameless creator the moves emptied is none)
  for creator in taken_away {
    if creator.get_name() == "creator"
      && document
        .findnode("ltx:personname", Some(&creator))
        .is_none()
      && element_nodes(&creator).is_empty()
    {
      document.remove_node(creator);
    }
  }
  Ok(())
}

/// Whether `node` prints text alone: every element under it a font wrapper (`ltx:text`) or an address's link (an
/// `ltx:ref` printing an `@`, `\url{{a,b}@x}`), so writing one of its addresses out ([`set_written_out`]) drops nothing
/// it prints — not an ORCID's `ltx:ref` (2308.02269's `\orcidID` glued to an address line, review rev100b B1), math,
/// a note.
fn prints_text_only(node: &Node) -> bool {
  element_nodes(node).iter().all(|child| {
    let address_link = child.get_name() == "ref" && child.get_content().contains('@');
    (child.get_name() == "text" || address_link) && prints_text_only(child)
  })
}

/// A copy of an e-mail contact printing several addresses made the contact of one, `address`: written in the font
/// wrapper the whole contact printed in (`<text font="typewriter">`, llncs's `\email`, 64zc), else as the contact's text.
fn set_written_out(contact: &Node, address: &str) {
  let mut node = contact.clone();
  loop {
    let children = node.get_child_nodes();
    let elements: Vec<&Node> = children
      .iter()
      .filter(|c| c.get_type() == Some(NodeType::ElementNode))
      .collect();
    let bare_text = children
      .iter()
      .any(|c| c.get_type() == Some(NodeType::TextNode) && !c.get_content().trim().is_empty());
    match elements.as_slice() {
      [only] if !bare_text => node = (*only).clone(),
      _ => break,
    }
  }
  let _ = node.set_content(address);
}

/// The author block's own creator (OD #159): the first nameless author creator after the named ones, else a new one
/// after the last of them.
fn block_creator(document: &mut Document, authors: &[Node]) -> Result<Option<Node>> {
  if let Some(creator) = document
    .findnodes("//ltx:creator[@role='author'][not(ltx:personname)]", None)
    .into_iter()
    .next()
  {
    return Ok(Some(creator));
  }
  let Some(mut last) = authors.last().cloned() else {
    return Ok(None);
  };
  let Some(parent) = last.get_parent() else {
    return Ok(None);
  };
  let saved = document.get_node().clone();
  document.set_node(&parent);
  let mut attributes = HashMap::default();
  attributes.insert("role".to_string(), "author".to_string());
  let mut creator = document.open_element("ltx:creator", Some(attributes), None)?;
  document.close_element("ltx:creator")?;
  document.set_node(&saved);
  if last.add_next_sibling(&mut creator).is_err() {
    return Ok(None);
  }
  Ok(Some(creator))
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
  // (each name read as printed, whom an address is moved to; and by its base letters, which only keep an address on
  // the author it spells so: `javier.martinez@ugr.es` spells Javier Martínez-López, its holder, by base letters, so it
  // stays, though as printed it spells Carlos Javier Navarro alone (brief 95c G6, 2410.00275); `muller-group@x.de` on
  // Anna Smith's own `\email` stays, though by base letters it spells Jörg Müller — no target is picked by base letters
  // (brief 95c r2 B1, FIDELITY_CHARTER §4 #420))
  let names: Vec<String> = authors
    .iter()
    .map(|author| {
      document
        .findnode("ltx:personname", Some(author))
        .map(|person| person.get_content())
        .unwrap_or_default()
    })
    .collect();
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(name))))
    .collect();
  let base_keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(&resolver::base_letters(name)))))
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
        && !spelled_owners(&base_keys, address).contains(&at)
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
///
/// A coalesce never drops a printed word: when no named author is left to take a nameless creator's annotations (a
/// block of affiliation lines alone, IEEEtran's `\authorblockA{…}` with no `\authorblockN`, 1804.00021; a name
/// written inside `\IEEEmembership{…}`, 2203.14581), the first nameless creator that carries any stays with every one
/// of them, the block's own creator — Perl's `<ltx:personname/>` over the lines (OXIDIZED_DESIGN_DIVERGENCES #159's
/// block owner, #949). Any other printing child of a removed creator is reported (`Warning:unexpected:frontmatter`), never
/// silently lost.
fn coalesce_empty_creators(document: &mut Document) -> Result<()> {
  let creators = document.findnodes("//ltx:creator[@role='author']", None);
  let mut to_remove: Vec<Node> = Vec::new();
  let mut last_real: Option<Node> = None;
  // Annotations (contacts + notes) of leading empties (before any real author), held
  // until the first real one; and the first of those empties that carries any, which keeps them all when no real
  // one follows.
  let mut orphan_annotations: Vec<Node> = Vec::new();
  let mut orphan_holder: Option<Node> = None;
  for creator in creators {
    if creator_personname_empty(document, &creator) {
      let annotations = creator_annotations(&creator);
      if let Some(ref mut prev) = last_real {
        if !annotations.is_empty() {
          document.append_clone(prev, annotations)?;
        }
        to_remove.push(creator);
      } else if orphan_holder.is_none() && !annotations.is_empty() {
        // (kept for now: the holder of the leading empties' annotations, if no real author takes them)
        orphan_holder = Some(creator);
      } else {
        orphan_annotations.extend(annotations);
        to_remove.push(creator);
      }
    } else {
      if let Some(holder) = orphan_holder.take() {
        // (the holder's own annotations first, in the order they stood)
        let held = creator_annotations(&holder);
        orphan_annotations.splice(0..0, held);
        to_remove.push(holder);
      }
      if !orphan_annotations.is_empty() {
        let mut first = creator.clone();
        document.append_clone(&mut first, std::mem::take(&mut orphan_annotations))?;
      }
      last_real = Some(creator);
    }
  }
  // (no named author took the leading empties' annotations: the holder keeps them all, the block's creator)
  if let Some(mut holder) = orphan_holder
    && !orphan_annotations.is_empty()
  {
    document.append_clone(&mut holder, orphan_annotations)?;
  }
  for creator in to_remove {
    // (what else a removed creator prints, no annotation moved with it, is reported: a coalesce never drops a word
    // silently)
    for child in creator.get_child_nodes() {
      if child.get_type() != Some(NodeType::ElementNode) {
        continue;
      }
      let qname = document::get_node_qname(&child);
      if with(qname, |q| {
        !matches!(q, "ltx:personname" | "ltx:contact" | "ltx:note" | "ltx:rdf")
      }) && !child.get_content().trim().is_empty()
      {
        Warn!(
          "unexpected",
          "frontmatter",
          s!(
            "A nameless author's {} was dropped with it: {}",
            with(qname, |q| q.to_string()),
            child.get_content().trim()
          )
        );
      }
    }
    document.remove_node(creator);
  }
  Ok(())
}

/// A creator's annotations, the children a coalesce moves: its `<ltx:contact>`, `<ltx:note>` and `<ltx:rdf>` elements.
fn creator_annotations(creator: &Node) -> Vec<Node> {
  creator
    .get_child_nodes()
    .into_iter()
    .filter(|c| {
      c.get_type() == Some(NodeType::ElementNode)
        && with(document::get_node_qname(c), |q| {
          q == "ltx:contact" || q == "ltx:note" || q == "ltx:rdf"
        })
    })
    .collect()
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
    // (what the author's entries answer and what its name shows, read where a request no entry answers asks: most
    // authors have none)
    let mut answered_here: Option<Vec<String>> = None;
    let mut shown: Option<(Vec<String>, Vec<String>)> = None;
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
      // (a symbol mark nothing answers is shown too — 1807.01584, 1805.02699, 2203.01270 — except one that placed one of
      // this author's contacts and was dropped from it, `$^\dagger$baker@y.edu`: answered on this author only,
      // `answered_symbol_key`; one the author's email shows as well stays on the name too, as printed: 1910.00401)
      let answered_here = answered_here.get_or_insert_with(|| {
        document
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
          .collect()
      });
      if symbol_mark_key(mark).is_some_and(|key| answered_here.contains(&key)) {
        continue;
      }
      let (shown, shown_sups) = shown.get_or_insert_with(|| {
        let sups = document
          .findnodes("ltx:personname//ltx:sup", Some(&creator))
          .iter()
          .map(|sup| sup.get_content())
          .collect();
        (shown_marks(document, &creator), sups)
      });
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
    let Some(mut personname) = document
      .findnodes("ltx:personname", Some(&creator))
      .into_iter()
      .next()
    else {
      continue;
    };
    let mut sup = Node::new("sup", None, document.get_document())?;
    if let Some(namespace) = personname.get_namespace() {
      let _ = sup.set_namespace(&namespace);
    }
    sup.append_text(&marks.join(","))?;
    // (marked as shown for no entry: the observer's writer takes from it the marks an item it writes answers — review
    // rev94a r2 S3)
    sup.set_attribute("_unanswered", "true")?;
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
    // (an annotation being digested — a contact's stub, `role=pending` — whose label is the mark its content requests
    // again is that one label: the relocation reads a stub's labels as one key, so `affiliation:1,affiliation:1` was no
    // creator's, an orphan on the nameless block creator — a rerouted `\affil{$^1$…}` given `label={affiliation:1}`
    // (frontmatter_stores.rs `contact_line_call`) digested under the author markup, pasa.cls:33, 2005.09266)
    let pending = attr.get("role").map(String::as_str) == Some("pending");
    let fresh: Vec<&str> = label
      .split(',')
      .filter(|piece| !(pending && labels.split(',').any(|known| known == *piece)))
      .collect();
    let label = fresh.join(",");
    let newval = if labels.is_empty() {
      label.clone()
    } else if label.is_empty() {
      labels
    } else {
      s!("{labels},{label}")
    };
    DebugFeature!("frontmatter", "FRONT add annotation label {label}");
    attr.insert("_annotations".to_string(), newval);
    if in_text && !label.is_empty() {
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
