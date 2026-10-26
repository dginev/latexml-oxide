//! Today's frontmatter as the document holds it once `insert_frontmatter` has placed it: per creator its name, and
//! its affiliations, contacts and notes, each with its node — read as `ab_audit fmshadow`'s `xml_tuples` reads the
//! XML (OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"), so that the writer compares what the comparator compares.

use super::*;

/// What an item is compared as: an affiliation (an `affiliation` or `address` contact), a note (an `ltx:note`, or a
/// contact of role `note`/`thanks`), or a contact of its role.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Kind {
  Affiliation,
  Contact(String),
  Note,
}

impl Kind {
  /// The kind a contact of `role` is compared as (`ab_audit fmshadow`'s `xml_tuples`).
  pub(super) fn of_contact(role: &str) -> Kind {
    match role {
      "affiliation" | "address" => Kind::Affiliation,
      "note" | "thanks" => Kind::Note,
      other => Kind::Contact(other.to_string()),
    }
  }

  /// The kind an item of the author block's own (the nameless creator) is compared as: `block_role`'s, a note of any
  /// role a note.
  pub(super) fn of_block(role: &str) -> Kind {
    match role {
      "affiliation" | "address" => Kind::Affiliation,
      "note" | "thanks" | "legend" | "footnotetext" => Kind::Note,
      other => Kind::Contact(other.to_string()),
    }
  }
}

/// One item of a creator: its kind, its text compared ([`norm`]) and its node; `movable` false for what the
/// personname holds (an ORCID link beside the name, a footnote set in it), which compares but is no entry of its own.
#[derive(Debug, Clone)]
pub(super) struct Item {
  pub kind:    Kind,
  pub key:     String,
  pub node:    Node,
  pub movable: bool,
}

/// One creator of today's frontmatter.
#[derive(Debug, Clone)]
pub(super) struct Creator {
  pub node:  Node,
  /// its personname's text without its marks
  pub name:  String,
  pub items: Vec<Item>,
}

/// Today's frontmatter: the named creators in document order, and the nameless ones (the author block's own, OD #159).
#[derive(Debug, Clone, Default)]
pub(super) struct Front {
  pub creators: Vec<Creator>,
  pub nameless: Vec<Creator>,
}

/// Text compared by its letters and digits alone, lowercased (`ab_audit fmshadow`'s `norm`).
pub(super) fn norm(text: &str) -> String {
  text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

fn is(node: &Node, qname: &str) -> bool {
  node.get_type() == Some(NodeType::ElementNode) && document::get_node_qname(node) == pin(qname)
}

/// The text a formula in a name or an affiliation prints when it is a mark (`ab_audit fmshadow`'s `math_mark_text`):
/// none for a floating superscript (`{}^{a,\;b}`, `^{1}`), the name for `\text{Leon Bello}^{1,3}`; `None` for any other
/// formula.
fn math_mark_text(math: &Node) -> Option<String> {
  let tex = math.get_attribute("tex")?;
  let mut tex = tex.trim();
  while let Some(rest) = [r"\,", r"\;", r"\!", r"\ ", "~", "{}"]
    .iter()
    .find_map(|p| tex.strip_prefix(p))
  {
    tex = rest.trim_start();
  }
  if tex.starts_with('^') {
    return Some(String::new());
  }
  for command in [r"\text{", r"\textrm{", r"\mbox{", r"\mathrm{"] {
    if let Some(rest) = tex.strip_prefix(command)
      && let Some(close) = rest.find('}')
      && rest[close + 1..].trim_start().starts_with('^')
    {
      return Some(rest[..close].to_string());
    }
  }
  None
}

/// A formula `<base>^{…}` whose base is a word or a number: that base (an affiliation's `OR $97403^{1}$`).
fn math_base(math: &Node) -> Option<String> {
  let tex = math.get_attribute("tex")?;
  let (base, _) = tex.trim().split_once('^')?;
  let base = base.trim();
  (!base.is_empty()
    && base
      .chars()
      .all(|c| c.is_alphanumeric() || c == '.' || c == '-'))
  .then(|| base.to_string())
}

/// A superscript of lowercase letters glued on both sides to letters (`James M<sup>c</sup>Kernan`, math0610203): part
/// of the word, as the lexer reads it (`events::lexer`).
fn inside_a_word(sup: &Node) -> bool {
  let text = sup.get_content();
  let glued = |side: Option<Node>, last: bool| {
    side
      .filter(|n| n.get_type() == Some(NodeType::TextNode))
      .is_some_and(|n| {
        let t = n.get_content();
        let c = if last {
          t.chars().next_back()
        } else {
          t.chars().next()
        };
        c.is_some_and(char::is_alphabetic)
      })
  };
  (1..=2).contains(&text.chars().count())
    && text.chars().all(char::is_lowercase)
    && glued(sup.get_prev_sibling(), true)
    && glued(sup.get_next_sibling(), false)
}

/// The text of `node` without its `ltx:sup` marks, its notes and their tags; a formula that is a mark as
/// [`math_mark_text`] reads it, an affiliation's (`scripts`) formula superscript a mark too.
pub(super) fn text_without_marks(node: &Node, scripts: bool) -> String {
  fn walk(node: &Node, scripts: bool, out: &mut String) {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::TextNode) => out.push_str(&child.get_content()),
        Some(NodeType::ElementNode) => {
          if is(&child, "ltx:Math")
            && let Some(text) =
              math_mark_text(&child).or_else(|| scripts.then(|| math_base(&child)).flatten())
          {
            out.push_str(&text);
          } else if (!is(&child, "ltx:sup") || inside_a_word(&child))
            && !is(&child, "ltx:note")
            && !is(&child, "ltx:tags")
          {
            // (a lowercase superscript inside a word is the word's letters, no mark: `M\textsuperscript{c}Kernan`)
            walk(&child, scripts, out);
          }
          if is(&child, "ltx:break") {
            out.push(' ');
          }
        },
        _ => {},
      }
    }
  }
  let mut raw = String::new();
  walk(node, scripts, &mut raw);
  raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// An ORCID link's identifier (`https://orcid.org/0000-0002-…` "0000-0002-…").
pub(super) fn orcid_id(text: &str) -> &str {
  let text = text.trim();
  ["https://orcid.org/", "http://orcid.org/", "orcid.org/"]
    .iter()
    .find_map(|p| text.strip_prefix(p))
    .unwrap_or(text)
}

fn descendants(node: &Node, qname: &str, out: &mut Vec<Node>) {
  for child in node.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) {
      if is(&child, qname) {
        out.push(child.clone());
      }
      descendants(&child, qname, out);
    }
  }
}

/// The ORCID links among `node`'s descendants, by their identifiers.
fn orcid_links(node: &Node) -> Vec<(Node, String)> {
  let mut refs = Vec::new();
  descendants(node, "ltx:ref", &mut refs);
  refs
    .into_iter()
    .filter_map(|r| {
      let href = r.get_attribute("href")?;
      href
        .contains("orcid.org/")
        .then(|| (r.clone(), orcid_id(&href).to_string()))
    })
    .collect()
}

/// One creator's items, as `xml_tuples` reads them.
fn read_creator(node: &Node) -> Creator {
  let mut names = Vec::new();
  let mut items = Vec::new();
  for child in node.get_child_nodes() {
    if is(&child, "ltx:personname") {
      names.push(text_without_marks(&child, false));
      // (a footnote set in the name: the creator's note; an ORCID link printed in it: its ORCID)
      let mut notes = Vec::new();
      descendants(&child, "ltx:note", &mut notes);
      for note in notes {
        let nested = {
          let mut up = note.get_parent();
          let mut inside = false;
          while let Some(parent) = up {
            if parent == child {
              break;
            }
            inside |= is(&parent, "ltx:note");
            up = parent.get_parent();
          }
          inside
        };
        if !nested {
          items.push(Item {
            kind:    Kind::Note,
            key:     norm(&text_without_marks(&note, false)),
            node:    note,
            movable: false,
          });
        }
      }
      for (link, id) in orcid_links(&child) {
        items.push(Item {
          kind:    Kind::Contact("orcid".into()),
          key:     norm(&id),
          node:    link,
          movable: false,
        });
      }
    } else if is(&child, "ltx:contact") {
      let role = child.get_attribute("role").unwrap_or_default();
      // (an ORCID link inside an e-mail contact is the creator's ORCID too)
      if role == "email" {
        for (link, id) in orcid_links(&child) {
          items.push(Item {
            kind:    Kind::Contact("orcid".into()),
            key:     norm(&id),
            node:    link,
            movable: false,
          });
        }
      }
      let kind = Kind::of_contact(&role);
      let mut text = text_without_marks(&child, kind == Kind::Affiliation);
      // (an ORCID contact prints a logo, its identifier the link's)
      if text.is_empty()
        && let Some(href) = {
          let mut refs = Vec::new();
          descendants(&child, "ltx:ref", &mut refs);
          refs.first().and_then(|r| r.get_attribute("href"))
        }
      {
        text = orcid_id(&href).to_string();
      }
      items.push(Item {
        kind,
        key: norm(&text),
        node: child.clone(),
        movable: true,
      });
    } else if is(&child, "ltx:note") {
      items.push(Item {
        kind:    Kind::Note,
        key:     norm(&text_without_marks(&child, false)),
        node:    child.clone(),
        movable: true,
      });
    }
  }
  Creator {
    node: node.clone(),
    name: names.join(" "),
    items,
  }
}

/// Today's frontmatter creators, in document order.
pub(super) fn read(document: &mut Document) -> Front {
  let mut front = Front::default();
  for node in document.findnodes("//ltx:creator", None) {
    let creator = read_creator(&node);
    if creator.name.trim().is_empty() {
      front.nameless.push(creator);
    } else {
      front.creators.push(creator);
    }
  }
  front
}
