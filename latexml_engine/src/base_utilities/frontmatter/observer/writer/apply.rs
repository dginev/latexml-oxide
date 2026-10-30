//! The writer's operations on today's frontmatter (FM S2b): copies of today's items for the authors the reading gives
//! them, the items made from queued tokens (a cut entry's pieces, a legend's note) digested as today's digests an
//! entry (`digest_frontmatter_item`) and inserted as today's inserts one (`insert_frontmatter_entry`), roles set, and
//! today's items taken away — in that order, so a copy is made before its source goes, and a copy that stays takes back
//! the ids of the one taken.

use std::collections::BTreeMap;

use super::{
  dom::{Front, read_creator},
  plan::{Key, Made, Op, Owner, Place, Plan},
  *,
};

/// The element children of `node`.
pub(super) fn elements(node: &Node) -> Vec<Node> {
  node
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect()
}

/// The author block's own creator (OD #159): today's first nameless creator, or a new one after the last creator.
fn block_creator(document: &mut Document, front: &Front) -> Result<Option<Node>> {
  if let Some(creator) = front.nameless.first() {
    return Ok(Some(creator.node.clone()));
  }
  let Some(mut last) = front.creators.last().map(|c| c.node.clone()) else {
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
  // (after the last creator, where today's puts the block's own)
  if last.add_next_sibling(&mut creator).is_err() {
    return Ok(None);
  }
  Ok(Some(creator))
}

/// The attributes an item made from `template` (today's contact it replaces) takes: its own, the internal ones aside.
fn template_attributes(template: &Node) -> TagAttrs {
  let mut attrs = TagAttrs::default();
  for (key, value) in template.get_attributes() {
    if !key.starts_with('_') && key != "id" && key != "xml:id" {
      attrs.insert(key, value);
    }
  }
  attrs
}

/// Digest `tokens` as `made` and insert it into `owner`, before `before` when given. An affiliation takes the
/// attributes of `template` (today's contact it replaces), or with none (the re-part's entry today's printed inside a
/// name) those of a contact of `role`: its role and its name label, as `\lx@add@affiliation` gives them.
pub(super) fn make(
  document: &mut Document,
  owner: &Node,
  made: Made,
  tokens: &Tokens,
  template: Option<&Node>,
  role: &str,
  before: Option<&Node>,
) -> Result<()> {
  let entry = match made {
    Made::Affiliation => {
      // (the wrapper today's affiliation calls digest their content in: `\inst` sets a label, the author markup)
      let wrapped = Invocation!(T_CS!("\\lx@affiliation@withinst"), vec![Some(
        tokens.clone()
      )]);
      let digested = digest_frontmatter_item("ltx:contact", wrapped)?;
      let attr = match template {
        Some(template) => template_attributes(template),
        None => {
          let mut attr = TagAttrs::default();
          attr.insert("role".to_string(), role.to_string());
          if let Some(name) = get_frontmatter_name(None, "ltx:contact", role)? {
            attr.insert("name".to_string(), name);
          }
          attr
        },
      };
      TagData {
        tag: "ltx:contact".to_string(),
        attr,
        content: vec![TagContent::Box(digested)],
      }
    },
    Made::Legend => {
      // (the kernel's author note, `\lx@add@thanks`: a marked note with its content kind; its mark's ink kept in its
      // text, the label-text ruling)
      let kind = classify_thanks(tokens.clone().untex_string().as_ref());
      let digested = digest_frontmatter_item("ltx:note", tokens.clone())?;
      let mut attr = TagAttrs::default();
      attr.insert("role".to_string(), "thanks".to_string());
      attr.insert(
        "class".to_string(),
        s!("ltx_note_frontmatter ltx_thanks_{kind}"),
      );
      TagData {
        tag: "ltx:note".to_string(),
        attr,
        content: vec![TagContent::Box(digested)],
      }
    },
  };
  let saved = document.get_node().clone();
  document.set_node(owner);
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
  let inserted = insert_frontmatter_entry(document, &entry);
  document.expire_box_to_absorb();
  document.set_node(&saved);
  inserted?;
  if let Some(before) = before
    && let Some(mut new) = elements(owner).pop()
  {
    let mut before = before.clone();
    let _ = before.add_prev_sibling(&mut new);
  }
  Ok(())
}

/// The element whose text a contact prints: the innermost of the chain of only children `node` opens (a contact's
/// `ltx:text` of the typewriter font), when it holds text alone — `None` when the chain holds anything else (a link,
/// a formula, two pieces), whose text the writer does not set.
pub(super) fn text_leaf(node: &Node) -> Option<Node> {
  let mut at = node.clone();
  loop {
    let children = at.get_child_nodes();
    let mut elements = children
      .iter()
      .filter(|c| c.get_type() != Some(NodeType::TextNode) || !c.get_content().trim().is_empty());
    match (elements.next(), elements.next()) {
      (Some(only), None)
        if only.get_type() == Some(NodeType::ElementNode)
          && document::get_node_qname(only) == pin("ltx:text") =>
      {
        at = only.clone();
      },
      _ => break,
    }
  }
  at.get_child_nodes()
    .iter()
    .all(|c| c.get_type() == Some(NodeType::TextNode))
    .then_some(at)
}

/// `node`'s text set to `text`, in its [`text_leaf`].
fn set_text(document: &Document, node: &Node, text: &str) -> Result<()> {
  let Some(mut at) = text_leaf(node) else {
    return Ok(());
  };
  for mut child in at.get_child_nodes() {
    child.unlink();
  }
  let mut node = Node::new_text(text, &document.document)?;
  at.add_child(&mut node)?;
  Ok(())
}

/// The elements of `node`'s subtree that carry an `xml:id`, with it, in document order.
pub(super) fn subtree_ids(document: &mut Document, node: &Node) -> Vec<(Node, String)> {
  document
    .findnodes("descendant-or-self::*[@xml:id]", Some(node))
    .into_iter()
    .filter_map(|n| n.get_attribute_ns("id", XML_NS).map(|id| (n, id)))
    .collect()
}

/// A taken item's ids given back to a copy of it that stays. Today's numbers each creator's copy of an item it places
/// under several (`footnotex3`, then `modify_id`'s `footnotex3a`), and a copy the writer makes is numbered so too: taking
/// the copy that held the plain id left the other's suffixed one standing alone (`footnotex3a`, no `footnotex3`). The
/// copy is the element named as the taken subtree's first id-holder whose id is that one's with a `radix_alpha` suffix
/// (a contact holds no id, the footnote mark in it does), its subtree's ids paired in order with the taken subtree's,
/// and references to them follow.
pub(super) fn reclaim_ids(document: &mut Document, taken: &[(SymStr, String)]) -> Result<()> {
  let Some(&(qname, ref id)) = taken.first() else {
    return Ok(());
  };
  if document.lookup_id(id).is_some() {
    return Ok(());
  }
  let Some(copy) = (1..=702).find_map(|n| {
    document
      .lookup_id(&s!("{id}{}", radix::radix_alpha(n)))
      .filter(|c| c.get_parent().is_some() && document::get_node_qname(c) == qname)
      .cloned()
  }) else {
    return Ok(());
  };
  let copy_ids = subtree_ids(document, &copy);
  if copy_ids.len() != taken.len() || taken.iter().any(|(_, id)| document.lookup_id(id).is_some()) {
    return Ok(());
  }
  for ((mut node, old), (_, new)) in copy_ids.into_iter().zip(taken) {
    document.unrecord_id(&old);
    document.set_attribute(&mut node, "xml:id", new)?;
    for mut reference in document.findnodes(&s!("//*[@idref='{old}']"), None) {
      document.set_attribute(&mut reference, "idref", new)?;
    }
  }
  Ok(())
}

/// `new`, an item just added to `owner` (the node of `who`), moved to where the reading prints it: before the first of
/// the owner's items the reading prints after it — not at the creator's end, where a copy made in the order of the
/// items' texts reordered what the PDF prints (rev92c S1: Sangyoun Lee's institutes 1 and 2 copied KIST before Yonsei,
/// 2207.06953; S. Mangla's ORIGINS before LMU and O. M. Smirnov's in the order of their names, 2608.05923; Ilyass
/// Haloui's written-out Airbus address after ISAE, 1812.02463). An item the reading places nowhere stays last.
/// Whether the reading places it.
pub(super) fn place(
  owner: &Node,
  who: Owner,
  new: &Node,
  order: &BTreeMap<(Owner, Key), Place>,
) -> bool {
  let items = read_creator(owner).items;
  // (within one print, an entry's own text before the contacts and notes it holds: `Univ X \\ \email{…}`)
  let place_of = |item: &dom::Item| {
    order
      .get(&(who, (item.kind.clone(), item.key.clone())))
      .map(|place| (*place, item.kind.clone()))
  };
  let Some(at) = items
    .iter()
    .find(|item| item.movable && item.node == *new)
    .and_then(place_of)
  else {
    return false;
  };
  if let Some(next) = items
    .iter()
    .filter(|item| item.movable && item.node != *new)
    .find(|item| place_of(item).is_some_and(|p| prints_after(&p, &at)))
  {
    let mut next = next.node.clone();
    let mut new = new.clone();
    let _ = next.add_prev_sibling(&mut new);
  }
  true
}

/// Whether the reading prints an item placed at `a` after one at `b`: a note's item after the author block's (aa's
/// `\offprints` e-mail below the institutes, review rev92c r2 B-1), then by their blocks or calls, then within one by
/// their spans — or, neither spanned, an entry's own text before what it holds (`Univ X \\ \email{…}`). One spanned
/// and one not in the same block are not ordered: a span is a position in the block, its absence none (an
/// `\institute`'s legend line cut at its span and the address line before it, which the row parted out with no span:
/// the legend had gone before the address it prints after, `perfect_kernel_batch61::marked_affiliation_review_shapes`).
fn prints_after(a: &(Place, dom::Kind), b: &(Place, dom::Kind)) -> bool {
  let ((anote, ai, (af, astart)), akind) = a;
  let ((bnote, bi, (bf, bstart)), bkind) = b;
  match ((anote, ai).cmp(&(bnote, bi)), af, bf) {
    (std::cmp::Ordering::Equal, 0, 0) => (astart, akind) > (bstart, bkind),
    (std::cmp::Ordering::Equal, 1, 1) => akind > bkind,
    (std::cmp::Ordering::Equal, ..) => false,
    (order, ..) => order == std::cmp::Ordering::Greater,
  }
}

/// Carry out `plan` on the document.
pub(super) fn apply(document: &mut Document, front: &Front, plan: &Plan) -> Result<()> {
  let mut block: Option<Node> = None;
  // (the owners the operations touch, whose items are then in the reading's order)
  let mut touched: Vec<(Owner, Node)> = Vec::new();
  let mut touch = |who: Owner, node: &Node| {
    if !touched.iter().any(|(o, _)| *o == who) {
      touched.push((who, node.clone()));
    }
  };
  let mut owner_node = |document: &mut Document, owner: Owner| -> Result<Option<Node>> {
    match owner {
      Owner::Creator(i) => Ok(plan.creators.get(&i).cloned()),
      Owner::Block => {
        if block.is_none() {
          block = block_creator(document, front)?;
        }
        Ok(block.clone())
      },
    }
  };
  for op in &plan.ops {
    if let Op::Copy { from, to } = op
      && let Some(mut owner) = owner_node(document, *to)?
    {
      document.append_clone(&mut owner, vec![from.clone()])?;
      touch(*to, &owner);
    }
  }
  for op in &plan.ops {
    if let Op::Make {
      owner: who,
      made,
      tokens,
      template,
      before,
    } = op
      && let Some(owner) = owner_node(document, *who)?
    {
      make(
        document,
        &owner,
        *made,
        tokens,
        Some(template),
        "affiliation",
        before.as_ref(),
      )?;
      touch(*who, &owner);
      // (an affiliation replacing none of today's items where the reading prints it, as a copy: 2311.16503's Jing Liu
      // had his institutes after the address the block prints below them; a legend stays last — the row's e-mail lines
      // are queued after the entry they print inside, so the calls' order does not place a legend line among them,
      // `perfect_kernel_batch61::marked_affiliation_review_shapes`)
      if before.is_none()
        && *made == Made::Affiliation
        && let Some(new) = elements(&owner).pop()
      {
        place(&owner, *who, &new, &plan.order);
      }
    }
  }
  for op in &plan.ops {
    if let Op::WriteOut { owner: who, template, address } = op
      && let Some(mut owner) = owner_node(document, *who)?
    {
      document.append_clone(&mut owner, vec![template.clone()])?;
      touch(*who, &owner);
      if let Some(mut copy) = elements(&owner).pop() {
        set_text(document, &copy, address)?;
        // (where the reading prints it; one it places nowhere at the template's place on the owner holding it, taken
        // after — today's place of a group may be no print order: Yuan Gao's `{malan,gaoy97}@` before his China
        // Telecom, 2603.23924)
        if !place(&owner, *who, &copy, &plan.order)
          && template.get_parent().as_ref() == Some(&owner)
        {
          let mut template = template.clone();
          let _ = template.add_prev_sibling(&mut copy);
        }
      }
    }
  }
  for op in &plan.ops {
    if let Op::Role { node, role } = op {
      let mut node = node.clone();
      document.set_attribute(&mut node, "role", role)?;
      if let Some(name) = get_frontmatter_name(None, "ltx:contact", role)? {
        document.set_attribute(&mut node, "name", &name)?;
      }
    }
  }
  // (the items taken, with their ids: a copy of one that stays takes them back)
  let mut taken = Vec::new();
  for op in &plan.ops {
    if let Op::Remove { node } = op {
      if let Some(parent) = node.get_parent() {
        if let Some((&i, _)) = plan.creators.iter().find(|(_, n)| **n == parent) {
          touch(Owner::Creator(i), &parent);
        } else if block.as_ref() == Some(&parent) || front.nameless.iter().any(|c| c.node == parent)
        {
          touch(Owner::Block, &parent);
        }
      }
      // (named while attached: a detached element's name no longer resolves its prefix)
      let ids: Vec<(SymStr, String)> = subtree_ids(document, node)
        .into_iter()
        .map(|(holder, id)| (document::get_node_qname(&holder), id))
        .collect();
      document.remove_node(node.clone());
      taken.push(ids);
    }
  }
  for ids in &taken {
    reclaim_ids(document, ids)?;
  }
  // (each touched owner's items in the reading's order, today's ones too: what an operation left — 2603.13900's Claudio
  // Di Ciccio kept today's address before his institute once the copy after it was taken; rev92c S1. Its notes stay
  // where they are, as a made legend does: the row's e-mail lines are queued after the entry they print inside, so the
  // calls' order does not place a note line among them)
  for (who, owner) in &touched {
    if owner.get_parent().is_some() {
      let items: Vec<Node> = read_creator(owner)
        .items
        .into_iter()
        .filter(|item| item.movable && item.kind != dom::Kind::Note)
        .map(|item| item.node)
        .collect();
      for item in items {
        place(owner, *who, &item, &plan.order);
      }
    }
  }
  // (a nameless creator the operations emptied is gone, as `coalesce_empty_creators` drops one)
  for creator in front.nameless.iter().map(|c| c.node.clone()).chain(block) {
    if creator.get_parent().is_some()
      && elements(&creator).iter().all(|e| {
        document::get_node_qname(e) == pin("ltx:personname") && e.get_content().trim().is_empty()
      })
    {
      document.remove_node(creator);
    }
  }
  Ok(())
}
