//! The placed frontmatter as the document holds it once `insert_frontmatter` has placed it: per creator its name, and
//! its affiliations, contacts and notes, each with its node ([`read`]) — what the content check reads
//! (`check_placed_content`) — and the printed text of a node without its marks ([`words_without_marks`]).

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
  /// The kind a contact of `role` is compared as (the deleted `ab_audit fmshadow`'s `xml_tuples`).
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
      "note" | "thanks" | "legend" | "footnotetext" | events::PICTURE_NOTE => Kind::Note,
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
  /// its text, every layout part parting the words on its sides ([`words_without_marks`]; its [`norm`] is `key`): what the
  /// content check counts the words of (`check_placed_content`)
  pub text:    String,
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

/// Today's frontmatter: the named creators in document order, the nameless ones (the author block's own, OD #159), and
/// the texts of the document's publication notes (`plan::held_as_pubnote`).
#[derive(Debug, Clone, Default)]
pub(super) struct Front {
  pub creators: Vec<Creator>,
  pub nameless: Vec<Creator>,
  pub pubnotes: Vec<String>,
}

// (text compared by its letters and digits alone, lowercased: the resolver's, which reads an entry's written span by it,
// `resolver::entry_span_in`)
pub(super) use super::super::resolver::norm;

pub(super) fn is(node: &Node, qname: &str) -> bool {
  node.get_type() == Some(NodeType::ElementNode) && document::get_node_qname(node) == pin(qname)
}

/// The text of the sibling just before `node` (`after`: just after it), when a text node.
fn sibling_text(node: &Node, after: bool) -> Option<String> {
  let side = if after {
    node.get_next_sibling()
  } else {
    node.get_prev_sibling()
  };
  side
    .filter(|n| n.get_type() == Some(NodeType::TextNode))
    .map(|n| n.get_content())
}

/// A superscript (an `ltx:sup`, or a formula that is a mark of one script) of lowercase letters glued on both sides to
/// letters (`James M<sup>c</sup>Kernan`, math0610203; `I. M. M$^{\rm c}$Hardy`, 2003.01448): part of the word, as the
/// lexer reads it (`events::lexer`; [`marks::glued_in_a_word`]).
fn inside_a_word(mark: &Node, letters: &str) -> bool {
  marks::glued_in_a_word(
    letters,
    sibling_text(mark, false).as_deref(),
    sibling_text(mark, true).as_deref(),
  )
}

/// The printed text of a number's suffix.
fn is_ordinal_suffix_text(letters: &str) -> bool {
  matches!(letters.trim(), "st" | "nd" | "rd" | "th")
}

/// A superscript printing a number's suffix after the number, outside a name (`events::lexer`'s `Event::ordinal`):
/// `5\textsuperscript{th}`, `201 E. 24$^{th}$ Street` (2202.05365), `his 60$^{th}$ birthday` (math9812087).
fn ordinal_suffix(mark: &Node, letters: &str) -> bool {
  is_ordinal_suffix_text(letters)
    && sibling_text(mark, false).is_some_and(|before| {
      before
        .trim_end()
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_digit())
    })
    && !in_a_personname(mark)
}

/// Whether `node` is inside an `ltx:personname`.
fn in_a_personname(node: &Node) -> bool {
  let mut up = node.get_parent();
  while let Some(parent) = up {
    if is(&parent, "ltx:personname") {
      return true;
    }
    up = parent.get_parent();
  }
  false
}

/// The text of `node` without its `ltx:sup` marks, its notes and their tags; a formula as [`marks::read_formula`]
/// reads it: a mark prints none (`{}^{a,\;b}`, `^{1}`, `\rm ^a`: revtex names `Dadhich${}^{a,\;b}$`, 1307.6166;
/// 2105.07958's `$\rm ^a$Email: …`), scripts on a base the base (`\text{Leon Bello}^{1,3}`, 2011.08099; an
/// affiliation's `OR $97403^{1}$`, `Ul. Ainy $299^A$`: 2203.01892, 0707.4047), any other formula its text (an isotope,
/// `${\rm ^{3}He}$`) — as the deleted `ab_audit fmshadow` reads the final XML.
pub(super) fn text_without_marks(node: &Node) -> String { printed_text(node, false) }

/// [`text_without_marks`], every cell, row, paragraph and box of a layout parting the words on its sides (a name in
/// one cell of a tabular and the institution in the next, which the comparison reads glued: what words a re-part keeps,
/// `repart`).
pub(super) fn words_without_marks(node: &Node) -> String { printed_text(node, true) }

/// Whether `node` holds an element of a layout parting the words on its sides ([`LAYOUT_PARTS`]): where its
/// [`words_without_marks`] differs from its [`text_without_marks`] by more than spaces.
pub(super) fn holds_a_layout_part(node: &Node) -> bool {
  node.get_child_nodes().iter().any(|child| {
    child.get_type() == Some(NodeType::ElementNode)
      && (LAYOUT_PARTS.iter().any(|q| is(child, q)) || holds_a_layout_part(child))
  })
}

/// The elements of a layout that part the words on their sides.
pub(super) const LAYOUT_PARTS: &[&str] = &[
  "ltx:td",
  "ltx:tr",
  "ltx:tabular",
  "ltx:p",
  "ltx:para",
  "ltx:inline-block",
  "ltx:inline-para",
  "ltx:block",
];

/// One piece of a node's printed text, in print order ([`segments`]).
pub(super) enum Seg {
  /// a text node, printed as it reads
  Text(Node),
  /// an element printed as one piece of text that no cut divides: a formula, a superscript inside a word
  Atom(String),
  /// an element that prints no text of the name: a mark, the tags of a note
  Hidden,
  /// a note set in the text: no text, but its own
  Note,
  /// the edge of a layout's part, or a line break or invisible ink ([`is_phantom`]): a space between the words
  /// on its sides
  Space,
}

impl Seg {
  /// The text the piece prints.
  pub(super) fn text(&self) -> String {
    match self {
      Seg::Text(node) => node.get_content(),
      Seg::Atom(text) => text.clone(),
      Seg::Space => " ".to_string(),
      Seg::Hidden | Seg::Note => String::new(),
    }
  }
}

/// The pieces `node` prints, in print order (a prescript before its base, as the deleted `ab_audit fmshadow` reads the parsed
/// XML): what [`text_without_marks`] reads — `parted`, every cell, row, paragraph and box of a layout a [`Seg::Space`]
/// on each side.
pub(super) fn segments(node: &Node, parted: bool) -> Vec<Seg> { segments_of(node, parted, false) }

/// [`segments`], `icons`: an element of no content (an `\includegraphics`, a `\rule`) an [`ICON`] atom — a piece
/// printing no text that no cut takes, for the address reader (brief 95c r3 F6).
fn segments_of(node: &Node, parted: bool, icons: bool) -> Vec<Seg> {
  fn walk(node: &Node, out: &mut Vec<Seg>, parted: bool, icons: bool, scripts: bool) {
    for child in marks::printed_children(node) {
      match child.get_type() {
        Some(NodeType::TextNode) => out.push(Seg::Text(child)),
        Some(NodeType::ElementNode) => {
          // (an undefined command's error markup, `<ltx:ERROR class="undefined">\cs</ltx:ERROR>` (Perl State.pm:549-551
          // `makeError`), prints none of the source's ink — TeX's "Undefined control sequence" prints nothing, as the
          // reading's events hold it: no word of the creator's, `Sanwoo Lee\internship` (2607.29185; OD #983))
          if is(&child, "ltx:ERROR") {
            out.push(Seg::Hidden);
          } else if is(&child, "ltx:Math") {
            match marks::read_formula(&child) {
              // (one lowercase script glued inside a word is its letters, as the `ltx:sup` the rewrite makes of it)
              marks::FormulaMarks::Marks(scripts) => match scripts.as_slice() {
                [letters] if inside_a_word(&child, letters) || ordinal_suffix(&child, letters) => {
                  let letters = letters.clone();
                  out.push(Seg::Atom(letters));
                },
                _ => out.push(Seg::Hidden),
              },
              // (`$4^{th}$`, the suffix of its base, as a script on a number is the reading's)
              marks::FormulaMarks::OnBase { base, scripts }
                if base.ends_with(|c: char| c.is_ascii_digit())
                  && matches!(scripts.as_slice(), [script] if is_ordinal_suffix_text(script))
                  && !in_a_personname(&child) =>
              {
                let text = s!("{base}{}", scripts[0]);
                out.push(Seg::Atom(text));
              },
              marks::FormulaMarks::OnBase { base, .. } => out.push(Seg::Atom(base)),
              // (its scripts its text too, one the binding kept as a superscript: REVTeX's `${\rm ^{3}He}$`)
              marks::FormulaMarks::Other => {
                let mut inner = Vec::new();
                walk(&child, &mut inner, parted, false, true);
                out.push(Seg::Atom(inner.iter().map(Seg::text).collect()));
              },
            }
          } else if is(&child, "ltx:sup") {
            // (a lowercase superscript inside a word is the word's letters, no mark: `M\textsuperscript{c}Kernan`; a
            // number's suffix is the text it is in, as the reading's `Event::ordinal` is, but a name's prefix `1st`)
            let content = child.get_content();
            if scripts || inside_a_word(&child, &content) || ordinal_suffix(&child, &content) {
              out.push(Seg::Atom(content));
            } else {
              out.push(Seg::Hidden);
            }
          } else if is(&child, "ltx:note") {
            out.push(Seg::Note);
          } else if is(&child, "ltx:tags") {
            out.push(Seg::Hidden);
          } else if is(&child, "ltx:break") || is_phantom(&child) {
            out.push(Seg::Space);
          } else if icons && child.get_first_child().is_none() {
            out.push(Seg::Atom(ICON.to_string()));
          } else {
            let part = parted && LAYOUT_PARTS.iter().any(|q| is(&child, q));
            if part {
              out.push(Seg::Space);
            }
            walk(&child, out, parted, icons, scripts);
            if part {
              out.push(Seg::Space);
            }
          }
        },
        _ => {},
      }
    }
  }
  let mut out = Vec::new();
  walk(node, &mut out, parted, icons, false);
  out
}

/// The text an icon prints for the address reader ([`segments_of`]): no letter, no separator, no space.
const ICON: char = '\u{FFFC}';

/// Whether `node` is invisible ink, `\phantom`, `\hphantom`, `\vphantom` in text (`<ltx:text class="ltx_phantom">`):
/// pdflatex prints a box of its argument's size and no ink, a space between the words beside it — no name's words
/// (review rev94a r2 B-2: 0909.3056's `\phantom{agga}` after the name read as an author; `field_ink::in_phantom`).
pub(super) fn is_phantom(node: &Node) -> bool {
  is(node, "ltx:text")
    && node
      .get_attribute("class")
      .is_some_and(|c| c.split_whitespace().any(|c| c == "ltx_phantom"))
}

/// The text of [`segments`], as printed.
fn raw_text(node: &Node, parted: bool) -> String {
  segments(node, parted).iter().map(Seg::text).collect()
}

fn printed_text(node: &Node, parted: bool) -> String {
  raw_text(node, parted)
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

/// An ORCID link's identifier (`https://orcid.org/0000-0002-…` "0000-0002-…").
pub(super) fn orcid_id(text: &str) -> &str {
  let text = text.trim();
  ["https://orcid.org/", "http://orcid.org/", "orcid.org/"]
    .iter()
    .find_map(|p| text.strip_prefix(p))
    .unwrap_or(text)
}

pub(super) fn descendants(node: &Node, qname: &str, out: &mut Vec<Node>) {
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
pub(super) fn read_creator(node: &Node) -> Creator {
  let mut names = Vec::new();
  let mut items = Vec::new();
  for child in node.get_child_nodes() {
    if is(&child, "ltx:personname") {
      names.push(text_without_marks(&child));
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
          let text = words_without_marks(&note);
          items.push(Item {
            kind: Kind::Note,
            key: norm(&text),
            text,
            node: note,
            movable: false,
          });
        }
      }
      for (link, id) in orcid_links(&child) {
        items.push(Item {
          kind:    Kind::Contact("orcid".into()),
          key:     norm(&id),
          text:    id,
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
            text:    id,
            node:    link,
            movable: false,
          });
        }
      }
      let kind = Kind::of_contact(&role);
      let mut text = words_without_marks(&child);
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
        text,
        node: child.clone(),
        movable: true,
      });
    } else if is(&child, "ltx:note") {
      let text = words_without_marks(&child);
      items.push(Item {
        kind: Kind::Note,
        key: norm(&text),
        text,
        node: child.clone(),
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
  for node in document.findnodes("//ltx:pubnote", None) {
    let key = norm(&text_without_marks(&node));
    if !key.is_empty() {
      front.pubnotes.push(key);
    }
  }
  front
}
