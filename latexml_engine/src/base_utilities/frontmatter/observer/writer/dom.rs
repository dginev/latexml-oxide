//! Today's frontmatter as the document holds it once `insert_frontmatter` has placed it: per creator its name, and
//! its affiliations, contacts and notes, each with its node — read as `ab_audit fmshadow`'s `xml_tuples` reads the
//! XML (OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"), so that the writer compares what the comparator compares.

use super::{
  super::resolver::{AddressRun, address_cuts, address_runs, addresses_in},
  *,
};

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

/// Today's frontmatter: the named creators in document order, the nameless ones (the author block's own, OD #159), and
/// the texts of the document's publication notes (`plan::held_as_pubnote`).
#[derive(Debug, Clone, Default)]
pub(super) struct Front {
  pub creators: Vec<Creator>,
  pub nameless: Vec<Creator>,
  pub pubnotes: Vec<String>,
}

/// Text compared by its letters and digits alone, lowercased (`ab_audit fmshadow`'s `norm`).
pub(super) fn norm(text: &str) -> String {
  text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

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
/// `${\rm ^{3}He}$`) — as `ab_audit fmshadow` reads the final XML.
pub(super) fn text_without_marks(node: &Node) -> String { printed_text(node, false) }

/// [`text_without_marks`], every cell, row, paragraph and box of a layout parting the words on its sides (a name in
/// one cell of a tabular and the institution in the next, which the comparison reads glued: what words a re-part keeps,
/// `repart`).
pub(super) fn words_without_marks(node: &Node) -> String { printed_text(node, true) }

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
  Atom(Node, String),
  /// an element that prints no text of the name: a mark, the tags of a note
  Hidden(Node),
  /// a note set in the text: no text, but its own
  Note(Node),
  /// the edge of a layout's part, or a line break or invisible ink (`Some`, [`is_phantom`]): a space between the words
  /// on its sides
  Space(Option<Node>),
}

impl Seg {
  /// The text the piece prints.
  pub(super) fn text(&self) -> String {
    match self {
      Seg::Text(node) => node.get_content(),
      Seg::Atom(_, text) => text.clone(),
      Seg::Space(_) => " ".to_string(),
      Seg::Hidden(_) | Seg::Note(_) => String::new(),
    }
  }
}

/// The marks `node` prints after its first ink, in print order: each hidden piece of [`segments`] (an `ltx:sup` not
/// inside a word, a formula that is a mark) some letter or digit precedes, its printed text (a formula's scripts each).
pub(super) fn later_marks(node: &Node) -> Vec<String> {
  let mut ink = false;
  let mut out = Vec::new();
  for seg in segments(node, false) {
    match seg {
      Seg::Text(text) => ink |= text.get_content().chars().any(char::is_alphanumeric),
      Seg::Atom(_, text) => ink |= text.chars().any(char::is_alphanumeric),
      Seg::Hidden(mark) if ink => {
        if is(&mark, "ltx:Math") {
          if let marks::FormulaMarks::Marks(scripts) = marks::read_formula(&mark) {
            out.extend(scripts);
          }
        } else if is(&mark, "ltx:sup") {
          out.push(mark.get_content());
        }
      },
      _ => {},
    }
  }
  out
}

/// The pieces `node` prints, in print order (a prescript before its base, as `ab_audit fmshadow` reads the parsed
/// XML): what [`text_without_marks`] reads — `parted`, every cell, row, paragraph and box of a layout a [`Seg::Space`]
/// on each side.
pub(super) fn segments(node: &Node, parted: bool) -> Vec<Seg> { segments_of(node, parted, false) }

/// [`segments`], `icons`: an element of no content (an `\includegraphics`, a `\rule`) an [`ICON`] atom — a piece
/// printing no text that no cut takes, for the address reader (brief 95c r3 F6).
fn segments_of(node: &Node, parted: bool, icons: bool) -> Vec<Seg> {
  fn walk(node: &Node, out: &mut Vec<Seg>, parted: bool, icons: bool) {
    for child in marks::printed_children(node) {
      match child.get_type() {
        Some(NodeType::TextNode) => out.push(Seg::Text(child)),
        Some(NodeType::ElementNode) => {
          if is(&child, "ltx:Math") {
            match marks::read_formula(&child) {
              // (one lowercase script glued inside a word is its letters, as the `ltx:sup` the rewrite makes of it)
              marks::FormulaMarks::Marks(scripts) => match scripts.as_slice() {
                [letters] if inside_a_word(&child, letters) || ordinal_suffix(&child, letters) => {
                  let letters = letters.clone();
                  out.push(Seg::Atom(child, letters));
                },
                _ => out.push(Seg::Hidden(child)),
              },
              // (`$4^{th}$`, the suffix of its base, as a script on a number is the reading's)
              marks::FormulaMarks::OnBase { base, scripts }
                if base.ends_with(|c: char| c.is_ascii_digit())
                  && matches!(scripts.as_slice(), [script] if is_ordinal_suffix_text(script))
                  && !in_a_personname(&child) =>
              {
                let text = s!("{base}{}", scripts[0]);
                out.push(Seg::Atom(child, text));
              },
              marks::FormulaMarks::OnBase { base, .. } => out.push(Seg::Atom(child, base)),
              marks::FormulaMarks::Other => {
                let text = raw_text(&child, parted);
                out.push(Seg::Atom(child, text));
              },
            }
          } else if is(&child, "ltx:sup") {
            // (a lowercase superscript inside a word is the word's letters, no mark: `M\textsuperscript{c}Kernan`; a
            // number's suffix is the text it is in, as the reading's `Event::ordinal` is, but a name's prefix `1st`)
            let content = child.get_content();
            if inside_a_word(&child, &content) || ordinal_suffix(&child, &content) {
              out.push(Seg::Atom(child, content));
            } else {
              out.push(Seg::Hidden(child));
            }
          } else if is(&child, "ltx:note") {
            out.push(Seg::Note(child));
          } else if is(&child, "ltx:tags") {
            out.push(Seg::Hidden(child));
          } else if is(&child, "ltx:break") || is_phantom(&child) {
            out.push(Seg::Space(Some(child)));
          } else if icons && child.get_first_child().is_none() {
            out.push(Seg::Atom(child, ICON.to_string()));
          } else {
            let part = parted && LAYOUT_PARTS.iter().any(|q| is(&child, q));
            if part {
              out.push(Seg::Space(None));
            }
            walk(&child, out, parted, icons);
            if part {
              out.push(Seg::Space(None));
            }
          }
        },
        _ => {},
      }
    }
  }
  let mut out = Vec::new();
  walk(node, &mut out, parted, icons);
  out
}

/// The text an icon prints for the address reader ([`segments_of`]): no letter, no separator, no space.
const ICON: char = '\u{FFFC}';

/// The marks of a name's superscript that `taken` reads as shown elsewhere, taken from its text nodes, each element
/// in it kept — a font wrapper inside it, `\textsuperscript{\textbf{2,*}}`: setting the superscript's content to the
/// marks kept dropped the bold (review rev94a r6 N1) —, and a comma left at either end of what stays. Whether a mark
/// is left.
pub(super) fn take_sup_marks(sup: &Node, taken: &dyn Fn(&str) -> bool) -> bool {
  fn texts(node: &Node, out: &mut Vec<Node>) {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::TextNode) => out.push(child),
        Some(NodeType::ElementNode) => texts(&child, out),
        _ => {},
      }
    }
  }
  let mut nodes = Vec::new();
  texts(sup, &mut nodes);
  for node in &mut nodes {
    let text = node.get_content();
    let kept: Vec<&str> = text.split(',').filter(|m| !taken(m.trim())).collect();
    if kept.len() < text.split(',').count() {
      let _ = node.set_content(&kept.join(","));
    }
  }
  let printed = |n: &Node| !n.get_content().trim().is_empty();
  if let Some(first) = nodes.iter_mut().find(|n| printed(n)) {
    let text = first.get_content();
    let _ = first.set_content(text.trim_start().trim_start_matches(','));
  }
  if let Some(last) = nodes.iter_mut().rev().find(|n| printed(n)) {
    let text = last.get_content();
    let _ = last.set_content(text.trim_end().trim_end_matches(','));
  }
  nodes.iter().any(|n| !n.get_content().trim().is_empty())
}

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

/// The e-mail addresses today's item prints that the reading takes out of an affiliation (brief 95c,
/// `Resolver::take_out_entry_addresses`), read by the reading's own lexer (`resolver::address_runs`) over the item's
/// printed text — its marks spaces, its line breaks line ends, as the reading's events part them: the key the item
/// compares as once they are cut (empty for an item of addresses alone), and each run's addresses ([`CutRun`]).
#[derive(Debug, Clone)]
pub(super) struct AddressCut {
  pub key:  String,
  pub runs: Vec<CutRun>,
}

/// One run of an [`AddressCut`]: each address with the wrapper it prints in, which the e-mail contact made of it keeps
/// ([`Wrap`]), and the label word the run took ("E-mail:"), the contact's name (B5).
#[derive(Debug, Clone)]
pub(super) struct CutRun {
  pub addresses: Vec<(String, Option<Wrap>)>,
  pub label:     Option<String>,
}

/// The wrapper an address cut out of an affiliation prints in, which the e-mail contact made of it keeps: `whole`, the
/// outermost inline element the cut takes whole — every piece under it text inside the cut ([`wraps_text`]), a link
/// in it kept only when it prints that one address or links to it: `\href{mailto:bob@y.org}{bob@y.org}` with its
/// `href`, `\texttt{ann@x.org}`, 0808.1112's `\emph{duan@iit.edu}` (2111.14566, 1804.04123; brief 95c r2 B4, r3 F1,
/// F3) —, else the innermost inline element it prints in, whose font alone the contact carries (a link holding an
/// icon, `\href{mailto:…}{\includegraphics{…} ann@x.org}`, or another address, `\href{mailto:able@x.org}{\{able,
/// baker\}@x.org}`, keeps its link and its icon where they printed: r3 F2, F3).
#[derive(Debug, Clone)]
pub(super) struct Wrap {
  pub node:  Node,
  pub whole: bool,
}

/// Whether `node` is an inline element that wraps text alone: no layout part, note, formula, line break or invisible
/// ink, its children text and such elements (`ltx:text`, `ltx:emph`, `ltx:ref`; an `\includegraphics` or a `\rule`
/// in it is none: brief 95c r3 F1, F2).
fn wraps_text(node: &Node) -> bool {
  inline_element(node)
    && node
      .get_child_nodes()
      .iter()
      .all(|child| match child.get_type() {
        Some(NodeType::TextNode) | Some(NodeType::CommentNode) => true,
        // (an element of no content is an icon, a rule: no text)
        Some(NodeType::ElementNode) => child.get_first_child().is_some() && wraps_text(child),
        _ => false,
      })
}

/// Whether `node` is an element printing inline text in a font of its own: no layout part, note, formula, mark, line
/// break or invisible ink.
fn inline_element(node: &Node) -> bool {
  node.get_type() == Some(NodeType::ElementNode)
    && !LAYOUT_PARTS.iter().any(|q| is(node, q))
    && ![
      "ltx:note",
      "ltx:Math",
      "ltx:break",
      "ltx:sup",
      "ltx:sub",
      "ltx:tags",
    ]
    .iter()
    .any(|q| is(node, q))
    && !is_phantom(node)
}

/// The printed text of `node` for the address reader: [`segments`], a mark, a note and a space a space, a line break a
/// `\n` (a line's end, as the reading's `Break` events part its lines: brief 95c r2 B2, N2), each segment's byte range
/// in it.
fn address_layout(node: &Node) -> (String, Vec<(usize, usize, Seg)>) {
  let mut text = String::new();
  let mut pieces = Vec::new();
  for seg in segments_of(node, false, true) {
    let printed = match &seg {
      Seg::Text(_) | Seg::Atom(..) => seg.text(),
      Seg::Space(Some(n)) if is(n, "ltx:break") => "\n".to_string(),
      Seg::Hidden(_) | Seg::Note(_) | Seg::Space(_) => " ".to_string(),
    };
    let start = text.len();
    text.push_str(&printed);
    pieces.push((start, text.len(), seg));
  }
  (text, pieces)
}

/// The node a piece of [`address_layout`] prints, if one.
fn seg_node(seg: &Seg) -> Option<&Node> {
  match seg {
    Seg::Text(n) | Seg::Atom(n, _) | Seg::Hidden(n) | Seg::Note(n) | Seg::Space(Some(n)) => Some(n),
    Seg::Space(None) => None,
  }
}

/// Whether `ancestor` is `node` or holds it.
fn holds(ancestor: &Node, node: &Node) -> bool {
  let mut at = Some(node.clone());
  while let Some(n) = at {
    if n == *ancestor {
      return true;
    }
    at = n.get_parent();
  }
  false
}

/// The cut of the addresses `node` prints ([`AddressCut`]) — only the runs every address of which `mailed` (the reading
/// holds it as an e-mail: a run it leaves in the entry stays) —, `None` when it prints none, or a cut would divide a
/// formula or take a note.
pub(super) fn address_cut(
  node: &Node,
  spells: &dyn Fn(&str) -> bool,
  mailed: &dyn Fn(&str) -> bool,
) -> Option<AddressCut> {
  let (text, pieces) = address_layout(node);
  let runs = mailed_runs(&text, spells, mailed);
  if runs.is_empty() {
    return None;
  }
  let cuts = address_cuts(&text, &runs);
  let mut kept = String::new();
  let mut at = 0;
  for &(start, end) in &cuts {
    for (ps, pe, seg) in &pieces {
      let overlaps = *ps < end && start < *pe;
      let inside = start <= *ps && *pe <= end;
      match seg {
        Seg::Atom(..) if overlaps && !inside => return None,
        Seg::Note(_) if overlaps => return None,
        _ => {},
      }
    }
    kept.push_str(&text[at..start]);
    at = end;
  }
  kept.push_str(&text[at..]);
  // (the wrapper the address `address` at `at` prints in, in the run `run`)
  let wrapper = |run: &AddressRun, at: usize, address: &str| -> Option<Wrap> {
    let (_, _, seg) = pieces.iter().find(|(ps, pe, _)| *ps <= at && at < *pe)?;
    let Seg::Text(leaf) = seg else {
      return None;
    };
    let (cs, ce) = cuts
      .iter()
      .copied()
      .find(|&(cs, ce)| cs <= run.start && run.end <= ce)?;
    // (a link kept whole only when it prints this one address or links to it: `\href{mailto:able@x.org}{\{able,
    // baker\}@x.org}` is no link of baker@'s, brief 95c r3 F3)
    let links_it = |link: &Node| {
      addresses_in(&link.get_content()).len() == 1
        || link
          .get_attribute("href")
          .and_then(|href| {
            href
              .get(..7)
              .filter(|scheme| scheme.eq_ignore_ascii_case("mailto:"))
              .map(|_| href[7..].trim().eq_ignore_ascii_case(address))
          })
          .unwrap_or(false)
    };
    let links_ok = |wrapper: &Node| {
      let mut refs = vec![wrapper.clone()];
      let mut at = 0;
      while at < refs.len() {
        let children = apply::elements(&refs[at]);
        refs.extend(children);
        at += 1;
      }
      refs.iter().filter(|n| is(n, "ltx:ref")).all(&links_it)
    };
    // (taken whole: an element wrapping text alone, every piece it prints text inside the cut — an icon, a rule, a
    // formula in it is not the address's: brief 95c r3 F2)
    let whole = |wrapper: &Node| {
      wraps_text(wrapper)
        && links_ok(wrapper)
        && pieces.iter().all(|(ps, pe, seg)| {
          seg_node(seg).is_none_or(|n| !holds(wrapper, n))
            || (matches!(seg, Seg::Text(_)) && cs <= *ps && *pe <= ce)
        })
    };
    let mut outermost = None;
    let mut font = None;
    let mut up = leaf.get_parent();
    while let Some(parent) = up {
      if parent == *node {
        break;
      }
      if whole(&parent) {
        outermost = Some(parent.clone());
      } else if font.is_none() && inline_element(&parent) {
        font = Some(parent.clone());
      }
      up = parent.get_parent();
    }
    // (a link kept for its font alone carries none the item does not: no wrapper then)
    let font = font.filter(|f| {
      !is(f, "ltx:ref")
        || f
          .get_attribute("_font")
          .is_some_and(|id| node.get_attribute("_font") != Some(id))
    });
    outermost
      .map(|node| Wrap { node, whole: true })
      .or(font.map(|node| Wrap { node, whole: false }))
  };
  Some(AddressCut {
    key:  norm(&kept),
    runs: runs
      .iter()
      .map(|r| {
        // (each address's own wrapper when the run prints as many `@` as it writes out addresses, else its first's)
        let ats: Vec<usize> = text[r.start..r.end]
          .match_indices('@')
          .map(|(i, _)| r.start + i)
          .collect();
        CutRun {
          addresses: r
            .addresses
            .iter()
            .enumerate()
            .map(|(i, a)| {
              let at = if ats.len() == r.addresses.len() {
                ats.get(i)
              } else {
                ats.first()
              };
              (a.clone(), at.and_then(|&at| wrapper(r, at, a)))
            })
            .collect(),
          label:     r.label.clone(),
        }
      })
      .collect(),
  })
}

/// The address runs of `text` every address of which `mailed`.
fn mailed_runs(
  text: &str,
  spells: &dyn Fn(&str) -> bool,
  mailed: &dyn Fn(&str) -> bool,
) -> Vec<AddressRun> {
  address_runs(text, spells)
    .into_iter()
    .filter(|run| run.addresses.iter().all(|a| mailed(a)))
    .collect()
}

/// Take the addresses [`address_cut`] reads out of `node`, in place: the text each cut covers taken from its text
/// nodes, the marks, formulas and line breaks inside it removed, and a font wrapper or a line break left with nothing to
/// print on its side removed too. Whether it cut.
pub(super) fn cut_addresses(
  node: &Node,
  spells: &dyn Fn(&str) -> bool,
  mailed: &dyn Fn(&str) -> bool,
) -> bool {
  if address_cut(node, spells, mailed).is_none() {
    return false;
  }
  let (text, pieces) = address_layout(node);
  let cuts = address_cuts(&text, &mailed_runs(&text, spells, mailed));
  for (ps, pe, seg) in &pieces {
    let mine: Vec<(usize, usize)> = cuts
      .iter()
      .filter(|(s, e)| s < pe && ps < e)
      .map(|&(s, e)| (s.max(*ps) - ps, e.min(*pe) - ps))
      .collect();
    if mine.is_empty() {
      continue;
    }
    match seg {
      Seg::Text(leaf) => {
        let content = leaf.get_content();
        let mut kept = String::new();
        let mut at = 0;
        for (s, e) in mine {
          if let (Some(head), Some(_)) = (content.get(at..s), content.get(s..e)) {
            kept.push_str(head);
            at = e;
          }
        }
        kept.push_str(content.get(at..).unwrap_or(""));
        let mut leaf = leaf.clone();
        let _ = leaf.set_content(&kept);
      },
      Seg::Atom(n, _) | Seg::Hidden(n) | Seg::Space(Some(n)) => {
        let mut n = n.clone();
        n.unlink();
      },
      _ => {},
    }
  }
  tidy_after_cut(node, cuts.last().is_some_and(|c| c.1 == text.len()));
  true
}

/// After a cut: a wrapper printing nothing removed (an inline element of text alone, [`wraps_text`]), a line break with
/// no printed text on one of its sides, and — the cut at the item's end — the separators it left there, up to the
/// first piece no text (an icon, a formula: brief 95c r3 F6).
fn tidy_after_cut(node: &Node, at_end: bool) {
  // (inner ones first: a link left holding an emptied font wrapper is empty once that goes)
  fn empty_wrappers(node: &Node) {
    for mut child in node.get_child_nodes() {
      if child.get_type() == Some(NodeType::ElementNode) {
        empty_wrappers(&child);
        // (an `ltx:ref` too: `\href{mailto:bob@y.org}{bob@y.org}` cut leaves no empty link, brief 95c r2 B4; an
        // `ltx:emph`, 0808.1112's `\emph{duan@iit.edu}`, r3 F1 — an empty element that held nothing, a line break,
        // an icon, stays)
        let wrapper = is(&child, "ltx:text")
          || is(&child, "ltx:ref")
          || (wraps_text(&child) && child.get_first_child().is_some());
        if wrapper && child.get_content().trim().is_empty() && apply::elements(&child).is_empty() {
          child.unlink();
        }
      }
    }
  }
  empty_wrappers(node);
  let pieces = segments_of(node, false, true);
  let printed =
    |seg: &Seg| matches!(seg, Seg::Text(_) | Seg::Atom(..)) && !seg.text().trim().is_empty();
  for (at, seg) in pieces.iter().enumerate() {
    if let Seg::Space(Some(n)) = seg
      && is(n, "ltx:break")
      && (!pieces[..at].iter().any(printed) || !pieces[at + 1..].iter().any(printed))
    {
      let mut n = n.clone();
      n.unlink();
    }
  }
  // (the separators a cut left at the item's end: `Trieste, Italy (…)` is `Trieste, Italy`)
  if !at_end {
    return;
  }
  // (the last piece printing: a text, or an element printing no text of its own — an icon, a rule, a formula, a mark:
  // `Univ X, \href{…}{$\boxtimes$\,ann@x.org}` keeps `Univ X, ⊠`, brief 95c r3 F6)
  fn last_piece(node: &Node) -> Option<Node> {
    for child in node.get_child_nodes().into_iter().rev() {
      match child.get_type() {
        Some(NodeType::TextNode) if !child.get_content().trim().is_empty() => return Some(child),
        // (a mark or a note prints none of the item's text; a formula, or an element of no content, an icon)
        Some(NodeType::ElementNode)
          if ["ltx:sup", "ltx:sub", "ltx:tags", "ltx:note"]
            .iter()
            .any(|q| is(&child, q)) => {},
        Some(NodeType::ElementNode)
          if is(&child, "ltx:Math") || child.get_first_child().is_none() =>
        {
          return Some(child);
        },
        Some(NodeType::ElementNode) => {
          if let Some(piece) = last_piece(&child) {
            return Some(piece);
          }
        },
        _ => {},
      }
    }
    None
  }
  if let Some(mut last) =
    last_piece(node).filter(|piece| piece.get_type() == Some(NodeType::TextNode))
  {
    let content = last.get_content();
    let trimmed = content.trim_end_matches(|c: char| {
      c.is_whitespace() || [',', ';', ':', '-', '\u{2013}', '\u{2014}', '/', '('].contains(&c)
    });
    if trimmed.len() < content.len() {
      let _ = last.set_content(trimmed);
    }
  }
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
          items.push(Item {
            kind:    Kind::Note,
            key:     norm(&text_without_marks(&note)),
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
      let mut text = text_without_marks(&child);
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
        key:     norm(&text_without_marks(&child)),
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
  for node in document.findnodes("//ltx:pubnote", None) {
    let key = norm(&text_without_marks(&node));
    if !key.is_empty() {
      front.pubnotes.push(key);
    }
  }
  front
}
