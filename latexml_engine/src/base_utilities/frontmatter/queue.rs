//! The front matter queue: queueing, document-set defaults, dequeueing and the paragraph breaks of a queued field.

use super::*;

/// While a raw class's own frontmatter setter replays for its side effects (`replay_class_setter`, OXIDIZED_DESIGN
/// #482), the frontmatter is the kernel's alone: nothing is queued (an entry, an annotation) or removed
/// (`\lx@clear@frontmatter`, which the kernel's `\lx@add@title`, `\lx@add@date`, `\lx@add@abstract` and
/// `\lx@add@keywords` run first), whatever path a body takes to the queue.
fn frontmatter_frozen_for_replay() -> bool { lookup_bool("lx_replaying_class_setter") }

/// frontmatter_raw contains the undigested commands to create frontmatter,
/// along with the tag & attributes that would be created.
/// Digestion is deferred until \maketitle, or something similar,
/// to avoid extra side-effects, particularly when entries
/// (eg. \title, \author) get redefined & replaced.
/// Use dequeue_front_matter or \lx@clear@frontmatter if there should be only 1 entry of $tag
/// Perl: queueFrontMatter($stomach, $tag, $attr, $command).
pub fn queue_front_matter(tag: &str, attr: Option<&KeyVals>, command: Tokens) {
  if frontmatter_frozen_for_replay() {
    return;
  }
  let queue = frontmatter_queue_name();
  note_document_sets_front_matter(
    tag,
    &attr
      .and_then(|kv| kv.get_value("role"))
      .map(|role| role.to_string())
      .unwrap_or_default(),
  );
  // Convert KeyVals to a hash, but be concerned about multiple values!?!?
  // (Perl: ToString($attr->getValue($_)) — the last value for multi-valued keys)
  let mut attr_hash = TagAttrs::default();
  if let Some(kv) = attr {
    for key in kv.get_keyvals().keys() {
      if let Some(value) = kv.get_value(key) {
        attr_hash.insert(key.clone(), value.to_string());
      }
    }
  }
  DebugFeature!(
    "frontmatter",
    "FRONT Queuing {tag} [{:?}] {command}",
    attr_hash
  );
  let missing = with_value(queue, |v| !matches!(v, Some(Stored::FrontmatterRaw(_))));
  if missing {
    assign_value(
      queue,
      Stored::FrontmatterRaw(Vec::new()),
      Some(Scope::Global),
    );
  }
  with_value_mut(queue, |val_opt| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
      queue.push((tag.to_string(), attr_hash, command));
    }
  });
}

/// The queue a frontmatter entry goes to: the document's (`frontmatter_raw`), or, while a stand-in binding's copy of
/// its author file is read (`binding::content::read_stand_in_copy`, OXIDIZED_DESIGN_DIVERGENCES #731), the copy's
/// defaults (`frontmatter_copy_defaults`): in TeX a class's own setter call (atlasdoc.cls:189 `\author{The ATLAS
/// Collaboration}`, :190 `\date{\today}`) only stores the default the document's own call replaces, which its title
/// page prints if the document makes none ([`hand_copy_frontmatter_defaults`]). A removal (`\lx@clear@frontmatter`,
/// which `\date` runs first) applies to the same queue: the copy's later `\date{}` (atlasdoc.cls:824 under
/// `cernpreprint`) clears its `\today`.
fn frontmatter_queue_name() -> &'static str {
  if definition::origin::copy_provides() != 0 {
    "frontmatter_copy_defaults"
  } else {
    "frontmatter_raw"
  }
}

/// A frontmatter entry `tag` of role `role` (`""`: every role) is set — queued or removed — outside a stand-in copy's
/// read ([`frontmatter_queue_name`]): the document's (or a binding's) setter call, whatever its value, replaces the
/// class default the copy stored, as `\def\@date{#1}` does in TeX ([`hand_copy_frontmatter_defaults`]): `\date{}`
/// removes the class's `\date{\today}`, and `\author{}`/`\title{}` theirs, though they queue nothing.
fn note_document_sets_front_matter(tag: &str, role: &str) {
  if definition::origin::copy_provides() != 0 {
    // (the copy stored a default: from now on its kernel field's setter is observed, as a binding's `\author` may
    // queue nothing for an empty call — inst_support's accumulates)
    if !definition::origin::setter_watch() {
      definition::origin::watch_setter_calls(&COPY_DEFAULT_SETTERS.map(|(setter, ..)| setter));
    }
  } else {
    assign_value(
      &s!("frontmatter_set:{tag}:{role}"),
      true,
      Some(Scope::Global),
    );
  }
}

/// The kernel's fields a stand-in copy's setter stores a default of: setter, tag, role, store.
const COPY_DEFAULT_SETTERS: [(&str, &str, &str, &str); 3] = [
  ("\\author", "ltx:creator", "author", "\\@author"),
  ("\\date", "ltx:date", "creation", "\\@date"),
  ("\\title", "ltx:title", "", "\\@title"),
];

/// Did the document set a frontmatter entry `tag` of role `role` — call its setter (`state::setter_called`, the
/// kernel's fields: whatever binding defines `\author`), or queue or remove one ([`note_document_sets_front_matter`];
/// a role-less removal sets every role)?
fn document_set_front_matter(tag: &str, role: &str) -> bool {
  lookup_bool(&s!("frontmatter_set:{tag}:{role}"))
    || (!role.is_empty() && lookup_bool(&s!("frontmatter_set:{tag}:")))
    || COPY_DEFAULT_SETTERS
      .iter()
      .any(|(setter, t, r, _)| *t == tag && *r == role && setter_called(&T_CS!(setter)))
}

/// At `\maketitle` (`\lx@store@defaults`): hand a stand-in copy's frontmatter defaults (`frontmatter_queue_name`) to
/// the document's frontmatter, each only when the document's setter made no call of its kind (tag and role: its own
/// `\author` replaces the class's, an empty `\date{}` too, `document_set_front_matter`), and, for the kernel's fields,
/// only when the copy's own title page prints the store (`frontmatter_stores::copy_title_page_prints`; the binding's
/// title page prints them when the copy defines none). Five ATLAS papers had the class's `<creator>` beside the
/// document's (1805.01649, 1806.07355, 1807.02012, 1902.05892, 1904.03536) and a `<date>` the class does not print
/// (1902.05892, 1904.03536).
pub fn hand_copy_frontmatter_defaults() {
  let defaults: Vec<RawFrontmatter> = match remove_value("frontmatter_copy_defaults") {
    Some(Stored::FrontmatterRaw(defaults)) => defaults,
    _ => return,
  };
  for (tag, attr, command) in defaults {
    let role = attr.get("role").cloned().unwrap_or_default();
    if document_set_front_matter(&tag, &role) || has_front_matter_with_role(&tag, &role) {
      continue;
    }
    let store = COPY_DEFAULT_SETTERS
      .iter()
      .find(|(_, t, r, _)| *t == tag && (*r == role || r.is_empty()))
      .map(|(.., store)| *store);
    if let Some(store) = store
      && crate::frontmatter_stores::copy_title_page_prints(&T_CS!(store)) == Some(false)
    {
      Info!(
        "frontmatter",
        tag,
        s!("The class's default {tag} is not printed by its title page: not shown")
      );
      continue;
    }
    let missing = with_value("frontmatter_raw", |v| {
      !matches!(v, Some(Stored::FrontmatterRaw(_)))
    });
    if missing {
      assign_value(
        "frontmatter_raw",
        Stored::FrontmatterRaw(Vec::new()),
        Some(Scope::Global),
      );
    }
    with_value_mut("frontmatter_raw", |val_opt| {
      if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
        queue.push((tag, attr, command));
      }
    });
  }
}

/// Is a frontmatter entry `tag` of role `role` (`""`: any) queued or digested ([`has_front_matter`])?
fn has_front_matter_with_role(tag: &str, role: &str) -> bool {
  let matches =
    |attr: &TagAttrs| role.is_empty() || attr.get("role").map(String::as_str) == Some(role);
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .any(|entry| entry.0 == tag && matches(&entry.1)),
    _ => false,
  });
  queued
    || with_value("frontmatter", |v| match v {
      Some(Stored::HashTagData(frnt)) => frnt
        .get(tag)
        .is_some_and(|list| list.iter().any(|e| matches(&e.attr))),
      _ => false,
    })
}

/// Build and queue the deferred `\lx@add@frontmatter@now` invocation for a
/// frontmatter entry `<tag attrs>content</tag>`. Shared by `\lx@add@frontmatter`
/// (content present) and `\lx@add@frontmatter@container` (content = None — a
/// deliberate empty container element that anchors later annotations).
/// Perl: queueFrontMatter($stomach, $tag, $attr,
///   Invocation(T_CS('\lx@add@frontmatter@now'), $keys, $tag, $attr, $tokens)).
pub(in crate::base_utilities) fn queue_add_frontmatter_now(
  keys_opt: Option<&KeyVals>,
  tag_tks: &Tokens,
  attrs_opt: Option<&KeyVals>,
  content: Option<&Tokens>,
) -> Result<()> {
  let mut inv_tokens: Vec<Token> = vec![T_CS!("\\lx@add@frontmatter@now")];
  if let Some(keys) = keys_opt {
    inv_tokens.push(T_OTHER!("["));
    inv_tokens.extend(keys.revert()?.unlist());
    inv_tokens.push(T_OTHER!("]"));
  }
  inv_tokens.push(T_BEGIN!());
  inv_tokens.extend(tag_tks.unlist_ref().iter().copied());
  inv_tokens.push(T_END!());
  if let Some(attrs) = attrs_opt {
    inv_tokens.push(T_OTHER!("["));
    inv_tokens.extend(attrs.revert()?.unlist());
    inv_tokens.push(T_OTHER!("]"));
  }
  inv_tokens.push(T_BEGIN!());
  if let Some(content) = content {
    inv_tokens.extend(content.unlist_ref().iter().copied());
  }
  inv_tokens.push(T_END!());
  queue_front_matter(&tag_tks.to_string(), attrs_opt, Tokens::new(inv_tokens));
  Ok(())
}

/// The number of creators queued so far (`frontmatter_raw`).
pub fn queued_creator_count() -> usize {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| entry.0 == "ltx:creator")
      .count(),
    _ => 0,
  })
}

/// Is `affiliation`'s first mark (its label, `entry_mark`) one a queued author carries — an affiliation number, not a
/// footnote symbol (`is_symbol_mark`: `$^*$Equal contribution` is the legend of the authors' stars)? Only then do the
/// authors showing that mark answer the affiliation: a mark no author carries (a typo, or authors without marks after
/// marked ones: `\author{Ann$^1$}\affil{$^1$X}\author{Bob}\affil{$^2$Y}`) would label an affiliation no one requests,
/// which the frontmatter places on an author of its own without a name. Nor a combined mark
/// (`\author{Ann$^{1}$}\author{Bob$^{2}$}\affil{$^{1,2}$Univ X}`, 77n r3 review probe w1): the affiliation is labelled
/// by its first label only (`mark_label`), so Bob lost it; it is the run's.
pub fn queued_authors_carry_entry_mark(affiliation: &Tokens) -> bool {
  entry_mark(affiliation.unlist_ref()).is_some_and(|mark| {
    !is_symbol_mark(&mark)
      && clean_frontmatter_labels(&Tokens::new(mark.clone()).to_string(), "affiliation").len() == 1
      && !requesting_authors(&queued_author_marks(), &mark).is_empty()
  })
}

/// Does a queued creator carry superscript marks (`\author{A\textsuperscript{1}}`) for affiliations
/// to be matched against?
pub fn queued_creators_have_marks() -> bool {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .any(|entry| entry.0 == "ltx:creator" && author_mark_position(&entry.2).is_some()),
    _ => false,
  })
}

/// Is a frontmatter item with this tag queued (`frontmatter_raw`) or already digested into the
/// frontmatter (`frontmatter`, which `digest_front_matter` fills at `\maketitle`; a pending
/// annotation stub is not an item)?
pub fn has_front_matter(tag: &str) -> bool {
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.iter().any(|entry| entry.0 == tag),
    _ => false,
  });
  queued
    || with_value("frontmatter", |v| match v {
      Some(Stored::HashTagData(frnt)) => frnt.get(tag).is_some_and(|list| {
        list
          .iter()
          .any(|e| e.attr.get("role").map(String::as_str) != Some("pending"))
      }),
      _ => false,
    })
}

/// This removes previously stored (but deferred) frontmatter that is being overridden.
/// It matches the tag and any stored attributes in `attr`.
/// Perl: dequeueFrontMatter($tag, %attr).
pub fn dequeue_front_matter(tag: &str, attr: &[(&str, &str)]) {
  if frontmatter_frozen_for_replay() {
    return;
  }
  note_document_sets_front_matter(
    tag,
    attr
      .iter()
      .find(|(key, _)| *key == "role")
      .map_or("", |(_, role)| role),
  );
  with_value_mut(frontmatter_queue_name(), |val_opt| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
      queue.retain(|entry| {
        let keep = entry.0 != tag
          || attr
            .iter()
            .any(|(k, v)| entry.1.get(*k).map(String::as_str).unwrap_or("") != *v);
        if !keep {
          DebugFeature!("frontmatter", "FRONT DEQueuing {} [{:?}]", entry.0, entry.1);
        }
        keep
      });
    }
  });
}

/// A `\par` digested in the frontmatter item `item` (its whatsit's `frontmatter_item`), constructed where the schema is
/// known: when the item's element holds only inline content (`ltx:acknowledgements`, `ltx:keywords`, `ltx:date`, …: no
/// `ltx:p`, but `ltx:break`) and is reached from the insertion point without passing an element that holds paragraphs
/// of its own, the paragraph break is a line break of the item, where text mode would merge the paragraphs (user ruling
/// 2026-10-18; apa.cls's two-paragraph `\acknowledgements`, OXIDIZED_DESIGN_DIVERGENCES #766). A break only parts
/// lines: none before the item's first content, none after another (a run of `\par`s), and those left at its end are
/// dropped when it closes ([`drop_trailing_paragraph_breaks`]). Returns whether the `\par` was the item's (it then
/// closes nothing); otherwise it is the ordinary `\par`.
pub(crate) fn frontmatter_paragraph_break(document: &mut Document, item: &str) -> Result<bool> {
  if document::can_contain_qname(item, "ltx:p") || !document::can_contain_qname(item, "ltx:break") {
    return Ok(false);
  }
  let Some(current) = document.get_element() else {
    return Ok(false);
  };
  let item_sym = pin(item);
  let mut open = Vec::new();
  let mut node = Some(current.clone());
  let item_node = loop {
    let Some(n) = node else {
      return Ok(false);
    };
    if document::get_node_qname(&n) == item_sym {
      break n;
    }
    if document::can_contain(&n, "ltx:p") || document::get_node_qname(&n) == pin!("ltx:p") {
      return Ok(false);
    }
    node = n.get_parent();
    open.push(n);
  };
  open.push(item_node.clone());
  if last_line_end(&item_node, &open) == Some(false) && document::can_contain(&current, "ltx:break")
  {
    document.insert_element(
      "ltx:break",
      Vec::new(),
      Some(map!(PARAGRAPH_BREAK_MARK.to_string() => "true".to_string())),
    )?;
  }
  Ok(true)
}

/// Note that the `\par`s digested from here on, at this mode frame, are paragraph breaks of the frontmatter field `tag`
/// ([`frontmatter_paragraph_break`]): a box, an alignment cell or math inside it begins a deeper mode frame, where a
/// `\par` is the ordinary one (in a box or cell it ends nothing, tex.web §1096 `end_graf`), while a group (`{\bfseries …}`)
/// begins none.
pub(in crate::base_utilities) fn note_frontmatter_paragraphs(tag: &str) {
  assign_value(
    "lx_frontmatter_par_item",
    tag.to_string(),
    Some(Scope::Local),
  );
  assign_value(
    "lx_frontmatter_par_depth",
    Stored::Int(mode_frame_depth()),
    Some(Scope::Local),
  );
}

/// How many frames bind the mode (`MODE`): each mode switch — a box, a cell, math, a frontmatter item's text — binds it
/// in a frame of its own.
pub(crate) fn mode_frame_depth() -> i64 { with_stacked_values("MODE", |modes| modes.len() as i64) }

/// The frontmatter field whose paragraph break a `\par` digested now is (what [`note_frontmatter_paragraphs`] noted, at
/// the same mode frame), if any.
pub(crate) fn frontmatter_paragraph_field() -> Option<String> {
  let item = lookup_string("lx_frontmatter_par_item");
  (!item.is_empty() && lookup_int("lx_frontmatter_par_depth") == mode_frame_depth()).then_some(item)
}

/// The transient mark on the `ltx:break`s [`frontmatter_paragraph_break`] makes (dropped with the other `_`
/// attributes when the document is finalized).
const PARAGRAPH_BREAK_MARK: &str = "_paragraph_break";

/// Does `node`'s content end with a paragraph break the frontmatter made (`Some(true)`), with other content
/// (`Some(false)`), or is it empty (`None`)? Whitespace counts for nothing, and so does an element still `open` (where
/// the content goes on) that holds nothing yet.
fn last_line_end(node: &Node, open: &[Node]) -> Option<bool> {
  let mut child = node.get_last_child();
  while let Some(n) = child {
    match n.get_type() {
      Some(NodeType::TextNode) if n.get_content().trim().is_empty() => {},
      Some(NodeType::ElementNode) if n.has_attribute(PARAGRAPH_BREAK_MARK) => return Some(true),
      Some(NodeType::ElementNode) => match last_line_end(&n, open) {
        Some(found) => return Some(found),
        None if open.iter().any(|o| o.to_hashable() == n.to_hashable()) => {},
        None => return Some(false),
      },
      _ => return Some(false),
    }
    child = n.get_prev_sibling();
  }
  None
}

/// Drop the paragraph breaks [`frontmatter_paragraph_break`] left at the end of the frontmatter element `node` (in its
/// last group too): a `\par` that ends an item ends no line of it.
pub(in crate::base_utilities) fn drop_trailing_paragraph_breaks(
  document: &mut Document,
  node: &Node,
) {
  let mut child = node.get_last_child();
  while let Some(n) = child {
    let previous = n.get_prev_sibling();
    match n.get_type() {
      Some(NodeType::TextNode) if n.get_content().trim().is_empty() => {},
      Some(NodeType::ElementNode) if n.has_attribute(PARAGRAPH_BREAK_MARK) => {
        document.remove_node(n)
      },
      Some(NodeType::ElementNode) => return drop_trailing_paragraph_breaks(document, &n),
      _ => return,
    }
    child = previous;
  }
}
