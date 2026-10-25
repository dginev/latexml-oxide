//! The writer's operations on today's frontmatter (FM S2b): copies of today's items for the authors the reading gives
//! them, the items made from queued tokens (a cut entry's pieces, a legend's note) digested as today's digests an
//! entry (`digest_frontmatter_item`) and inserted as today's inserts one (`insert_frontmatter_entry`), roles set, and
//! today's items taken away — in that order, so a copy is made before its source goes, and a copy that stays takes back
//! the ids of the one taken.

use super::{
  dom::Front,
  plan::{Made, Op, Owner, Plan},
  *,
};

/// The element children of `node`.
fn elements(node: &Node) -> Vec<Node> {
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

/// Digest `tokens` as `made` and insert it into `owner`, before `before` when given.
fn make(
  document: &mut Document,
  owner: &Node,
  made: Made,
  tokens: &Tokens,
  template: &Node,
  before: Option<&Node>,
) -> Result<()> {
  let entry = match made {
    Made::Affiliation => {
      // (the wrapper today's affiliation calls digest their content in: `\inst` sets a label, the author markup)
      let wrapped = Invocation!(T_CS!("\\lx@affiliation@withinst"), vec![Some(
        tokens.clone()
      )]);
      let digested = digest_frontmatter_item("ltx:contact", wrapped)?;
      TagData {
        tag:     "ltx:contact".to_string(),
        attr:    template_attributes(template),
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

/// The elements of `node`'s subtree that carry an `xml:id`, with it, in document order.
fn subtree_ids(document: &mut Document, node: &Node) -> Vec<(Node, String)> {
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
fn reclaim_ids(document: &mut Document, taken: &[(SymStr, String)]) -> Result<()> {
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

/// Carry out `plan` on the document.
pub(super) fn apply(document: &mut Document, front: &Front, plan: &Plan) -> Result<()> {
  let mut block: Option<Node> = None;
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
    }
  }
  for op in &plan.ops {
    if let Op::Make {
      owner,
      made,
      tokens,
      template,
      before,
    } = op
      && let Some(owner) = owner_node(document, *owner)?
    {
      make(document, &owner, *made, tokens, template, before.as_ref())?;
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
