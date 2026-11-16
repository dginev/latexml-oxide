//! A note several creators share, as a class prints it once under one mark (DG rulings of 2026-11-16: the XML keeps a
//! full copy on every carrier, each with `mark`; the XSLT prints it once): a class's footnote text two or more names
//! carry under one mark, and the names' own copy of the mark a title-page list's note carries
//! (OXIDIZED_DESIGN_DIVERGENCES #1054).

use super::*;

/// The attribute of a title-page list's note that the names showing its leading mark carry (`\lx@fm@list@note`): that
/// mark.
pub(in crate::base_utilities) const LIST_MARK: &str = "_listmark";
/// The attribute of a creator's contact that is a class's footnote text: the mark the class prints for it (elsarticle's
/// `\fntext`, REVTeX's `\thanks`).
pub(in crate::base_utilities) const PRINTED_MARK: &str = "_fnmark";

/// The Kind step for a class's footnote text that two or more creators carry under one mark ([`PRINTED_MARK`]): an
/// elsarticle `\fntext[eq]{…}` its `\fnref{eq}` gives two names (each copy a contact `relocate_annotations` placed), a
/// REVTeX `\thanks{…}` two names give alike — printed once, its mark on each name (elsarticle.cls:157-171, revtex4-2.cls:2472-2495
/// `\frontmatter@footnote`): each copy the carriers' note with that `mark` (FIDELITY_CHARTER §4, the mark rule). One a
/// single creator carries stays its contact, shown under its name.
pub(super) fn kind_shared_marked_contacts(document: &mut Document) -> Result<()> {
  let contacts = document.findnodes(&s!("//ltx:creator/ltx:contact[@{PRINTED_MARK}]"), None);
  if contacts.is_empty() {
    return Ok(());
  }
  // (each copy's mark and text; a copy is shared when two creators or more carry its mark and text)
  let read: Vec<(String, String)> = contacts
    .iter()
    .map(|contact| {
      (
        contact.get_attribute(PRINTED_MARK).unwrap_or_default(),
        contact
          .get_content()
          .split_whitespace()
          .collect::<Vec<_>>()
          .join(" "),
      )
    })
    .collect();
  let mut carriers: HashMap<&(String, String), Vec<Node>> = HashMap::default();
  for (contact, copy) in contacts.iter().zip(&read) {
    let held = carriers.entry(copy).or_default();
    if let Some(creator) = contact.get_parent()
      && !held.contains(&creator)
    {
      held.push(creator);
    }
  }
  let shared: Vec<bool> = read
    .iter()
    .map(|copy| !copy.0.is_empty() && carriers[copy].len() >= 2)
    .collect();
  for (at, mut contact) in contacts.into_iter().enumerate() {
    contact.remove_attribute(PRINTED_MARK).ok();
    if !shared[at] {
      continue;
    }
    let kind = classify_thanks(&read[at].1);
    let mut note = document.rename_node(contact, "ltx:note", true)?;
    note.remove_attribute("name").ok();
    document.set_attribute(&mut note, "role", "thanks")?;
    document.set_attribute(
      &mut note,
      "class",
      &s!("ltx_note_frontmatter ltx_thanks_{kind}"),
    )?;
    document.set_attribute(&mut note, "mark", &read[at].0)?;
  }
  Ok(())
}

/// The superscript (or the formula of one) the text of `node` opens with, past the paragraphs and the font wrappers that
/// open it, and what it prints: `<p align="center"><sup>‡</sup> These authors …</p>`.
fn leading_mark(node: &Node) -> Option<(Node, String)> {
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) if child.get_content().trim().is_empty() => {},
      Some(NodeType::ElementNode) => {
        return match child.get_name().as_str() {
          "sup" => Some((child.clone(), child.get_content().trim().to_string())),
          "Math" => match fm_read_formula(&child) {
            FmFormulaMarks::Marks(scripts) if scripts.len() == 1 => {
              Some((child.clone(), scripts[0].trim().to_string()))
            },
            _ => None,
          },
          "p" | "para" | "text" | "inline-block" | "inline-para" => leading_mark(&child),
          _ => None,
        };
      },
      _ => return None,
    }
  }
  None
}

/// A title-page list's note its leading mark gave to the names showing it ([`LIST_MARK`], `\lx@fm@list@note`: 2606.06621's
/// `$^\ddagger$ These authors contributed equally` under `Windt$^\ddagger$`) shows that mark as its `mark`: the mark goes
/// from its text, with the space after it (the mark rule, FIDELITY_CHARTER §4), and from its holder's name, as a name's
/// mark a legend answers does (`show_unanswered_marks`). A name's superscript of other marks too (`$^{1,\ddagger}$`)
/// stays.
pub(super) fn show_list_note_marks(document: &mut Document) -> Result<()> {
  let key = |mark: &str| mark.trim().replace('∗', "*");
  for mut note in document.findnodes(&s!("//ltx:creator/ltx:note[@{LIST_MARK}]"), None) {
    let mark = note.get_attribute(LIST_MARK).unwrap_or_default();
    note.remove_attribute(LIST_MARK).ok();
    if let Some((lead, printed)) = leading_mark(&note)
      && key(&printed) == key(&mark)
    {
      // (the space after it: a text node, or the first of the font wrapper after it)
      let next = lead
        .get_next_sibling()
        .and_then(|next| match next.get_type() {
          Some(NodeType::TextNode) => Some(next),
          _ if next.get_name() == "text" => next
            .get_first_child()
            .filter(|n| n.get_type() == Some(NodeType::TextNode)),
          _ => None,
        });
      document.remove_node(lead);
      if let Some(mut text) = next {
        let content = text.get_content();
        text.set_content(content.trim_start())?;
      }
    }
    let Some(creator) = note.get_parent() else {
      continue;
    };
    for sup in document.findnodes("ltx:personname//ltx:sup", Some(&creator)) {
      if key(&sup.get_content()) == key(&mark) {
        document.remove_node(sup);
      }
    }
  }
  Ok(())
}
