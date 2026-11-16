//! The name alone: the marks printed on a creator's name re-homed outside its `ltx:personname` (FIDELITY_CHARTER
//! "a name's label is the name alone", 2026-10-21; DG 2026-11-16: purity and no lost mark are one rule).

use super::{
  insert::{MARK_FROM_COUNTER, drop_emptied_wrappers, mark_form},
  observer::marks::glued_in_a_word,
  *,
};
use crate::base_utilities::defs::frontmatter_api::PRINTED_LABEL;

/// The footnote symbols a name's end may carry typed by hand (`Yuan Yuan*`, 2007.16032; `Shicheng Wang†`,
/// math0304301; `D. Hernández Ruipérez¶`, alg-geom9405006).
const HAND_MARKS: [char; 7] = [
  '*', '\u{2217}', '\u{2020}', '\u{2021}', '\u{A7}', '\u{B6}', '#',
];

/// Takes every mark off each creator's personname: its note's `mark`, the label of a contact it holds (consumed), or a
/// mark-only note after the name; never dropped. OXIDIZED_DESIGN_DIVERGENCES #1055, guards `perfect_kernel_batchpure`.
pub(in crate::base_utilities) fn keep_personnames_pure(document: &mut Document) -> Result<()> {
  for creator in document.findnodes("//ltx:creator[ltx:personname]", None) {
    for personname in children_named(&creator, "personname") {
      let groups = mark_groups(&personname);
      if groups.is_empty() {
        continue;
      }
      let held: HashSet<String> = children_named(&creator, "contact")
        .iter()
        .flat_map(contact_keys)
        .collect();
      let notes: Vec<Node> = children_named(&creator, "note")
        .into_iter()
        .filter(|note| note.get_attribute("role").as_deref() != Some("footnotemark"))
        .collect();
      let mut after = personname.clone();
      let mut parents: Vec<Node> = Vec::new();
      for group in groups {
        let printed = group.printed.trim().to_string();
        let pieces = mark_pieces(&printed);
        let mut left: Vec<String> = Vec::new();
        let mut taken = false;
        for piece in &pieces {
          let key = mark_form(piece);
          let keys: Vec<String> = std::iter::once(key.clone())
            .chain(group.refmarks.iter().cloned())
            .collect();
          if answer(document, &notes, &held, piece, &key, &keys)? {
            taken = true;
            continue;
          }
          // (a run of glued labels: each a held contact's, `$^{ab}$` over `$^a$Univ A`, `$^b$Univ B`, revtex3's; or a
          // label glued to a note's number, `$^{a1}$` over `$^a$Department …` (hep-ph0011101), the number left)
          let held_part = |part: &str| held.contains(&mark_form(part));
          let glued: Vec<String> = if piece.chars().count() > 1
            && piece.chars().all(char::is_alphanumeric)
            && piece.chars().all(|c| held_part(&c.to_string()))
          {
            Vec::new()
          } else {
            let parts = letter_digit_runs(piece);
            if parts.len() > 1 && parts.iter().any(|part| held_part(part)) {
              parts.into_iter().filter(|part| !held_part(part)).collect()
            } else {
              left.push(piece.clone());
              continue;
            }
          };
          taken = true;
          left.extend(glued);
        }
        let shown = left.join(",");
        DebugFeature!(
          "frontmatter",
          "FRONT pure name: mark {printed}, {shown} left outside the name"
        );
        group.cut(document, &mut parents);
        if !left.is_empty() {
          // (as printed where nothing was answered, `(a,b)`, or a range was printed, `1-3`; else the marks left, a list)
          let ranged = printed.contains(['-', '\u{2013}', '\u{2212}']);
          let mark = if !taken || ranged {
            printed.clone()
          } else {
            left.join(",")
          };
          let mut note = mark_only_note(document, &personname, &mark)?;
          after.add_next_sibling(&mut note)?;
          after = note;
        }
      }
      drop_emptied_wrappers(document, parents);
      trim_name_edges(document, &personname);
    }
  }
  Ok(())
}

/// Whether `piece` (keyed `key`; `keys` with an author reference mark's numbers, `\IEEEauthorrefmark{4}` printing `§`)
/// is answered on its creator: a note marked so; a contact keyed so (`held`); or an unmarked note whose text opens with
/// a copy of it (a legend: `\thanks{*Equal Contribution}`, 1904.03977), which takes it as its `mark` ([`leading_mark`]).
/// A symbol beside a `\thanks` is no `\thanks`'s mark: the `\thanks` prints its own (`Haoran Li$^{*}$\thanks{…}` prints
/// "∗†", 2406.11149), the same glyph too (`Ann Able*\thanks{…}` prints "*∗": a note its command marks,
/// [`MARK_FROM_COUNTER`], answers no typed glyph).
fn answer(
  document: &mut Document,
  notes: &[Node],
  held: &HashSet<String>,
  piece: &str,
  key: &str,
  keys: &[String],
) -> Result<bool> {
  if notes.iter().any(|note| {
    note.get_attribute(MARK_FROM_COUNTER).is_none()
      && note
        .get_attribute("mark")
        .is_some_and(|mark| mark_form(&mark) == key)
  }) {
    return Ok(true);
  }
  if keys.iter().any(|key| held.contains(key)) {
    return Ok(true);
  }
  for note in notes {
    if note
      .get_attribute("mark")
      .is_some_and(|mark| !mark.trim().is_empty())
    {
      continue;
    }
    if let Some(lead) = leading_mark(note, piece) {
      let mut note = note.clone();
      document.set_attribute(&mut note, "mark", piece)?;
      lead.cut(document);
      return Ok(true);
    }
  }
  Ok(false)
}

/// The runs of letters and of digits `piece` parts into (`a1`: `a`, `1`), each its own mark where one is a label.
fn letter_digit_runs(piece: &str) -> Vec<String> {
  let mut runs: Vec<String> = Vec::new();
  let mut last_digit = None;
  for c in piece.chars() {
    let digit = c.is_ascii_digit();
    match runs.last_mut() {
      Some(run) if last_digit == Some(digit) => run.push(c),
      _ => runs.push(c.to_string()),
    }
    last_digit = Some(digit);
  }
  runs
}

/// The element children of `node` named `name` (in the LaTeXML namespace).
fn children_named(node: &Node, name: &str) -> Vec<Node> {
  node
    .get_child_nodes()
    .into_iter()
    .filter(|child| child.get_type() == Some(NodeType::ElementNode) && child.get_name() == name)
    .collect()
}

/// The mark keys a contact is placed by or prints ([`mark_form`]): its labels (`_label`, `_bymark`, the printed label
/// of an emitted entry, [`PRINTED_LABEL`]) — an affiliation's, or one of no kind, a command's spelling as its glyph
/// (`\dag`: †, [`unanswered_mark_text`]) — and the superscript its text opens with (`$^1$Raman Research Institute`,
/// 0707.3527), inside its font wrappers too.
fn contact_keys(contact: &Node) -> Vec<String> {
  let mut keys = Vec::new();
  for attribute in ["_label", "_bymark", PRINTED_LABEL] {
    let Some(labels) = contact.get_attribute(attribute) else {
      continue;
    };
    for label in clean_frontmatter_labels(&labels, "") {
      let key = match label.split_once(':') {
        Some(("affiliation", key)) => key,
        Some(_) => continue,
        None => label.as_str(),
      };
      let key = unanswered_mark_text(key).unwrap_or_else(|| key.to_string());
      keys.push(mark_form(&key));
    }
  }
  // (through the font or box wrappers it opens with: authblk's `\textsuperscript{\(*\)}`, `<text><sup>…`)
  let first = |node: &Node| {
    node.get_child_nodes().into_iter().find(|child| {
      child.get_type() != Some(NodeType::TextNode) || !child.get_content().trim().is_empty()
    })
  };
  let mut opening = first(contact);
  while let Some(node) = opening
    .as_ref()
    .filter(|node| node.get_type() == Some(NodeType::ElementNode) && node.get_name() == "text")
  {
    opening = first(node);
  }
  let printed = opening
    .filter(|node| node.get_type() == Some(NodeType::ElementNode))
    .and_then(|node| match node.get_name().as_str() {
      "sup" => Some(node.get_content().trim().to_string()),
      "Math" => match fm_read_formula(&node) {
        FmFormulaMarks::Marks(scripts) => Some(scripts.join(",")),
        _ => None,
      },
      _ => None,
    });
  if let Some(printed) = printed {
    keys.extend(mark_pieces(&printed).iter().map(|piece| mark_form(piece)));
  }
  keys.retain(|key| !key.is_empty());
  keys
}

/// The marks of a printed mark run, in order: its items at commas and spaces, out of their brackets (`(a,b,∗)`), each
/// parted further where a letter or number run meets a symbol and between distinct symbols (`1∗`, `*‡`), a run of one
/// symbol one mark (`**`); a range's dash parts its ends.
fn mark_pieces(printed: &str) -> Vec<String> {
  let mut pieces = Vec::new();
  let mut current = String::new();
  let flush = |current: &mut String, pieces: &mut Vec<String>| {
    if !current.is_empty() {
      pieces.push(std::mem::take(current));
    }
  };
  for c in printed.chars() {
    if c.is_whitespace()
      || matches!(
        c,
        ',' | ';' | '(' | ')' | '[' | ']' | '-' | '\u{2013}' | '\u{2212}'
      )
    {
      flush(&mut current, &mut pieces);
      continue;
    }
    if is_combining_mark(c) {
      current.push(c);
      continue;
    }
    let last = current.chars().rev().find(|c| !is_combining_mark(*c));
    let parts = last.is_some_and(|last| {
      if last.is_alphanumeric() {
        !c.is_alphanumeric()
      } else {
        c.is_alphanumeric() || c != last
      }
    });
    if parts {
      flush(&mut current, &mut pieces);
    }
    current.push(c);
  }
  flush(&mut current, &mut pieces);
  pieces
}

/// One run of marks on a name, as printed: its superscripts and the blank text between them, or the footnote symbols
/// typed at the end of a text (cut at `hand`).
struct MarkGroup {
  printed:  String,
  /// its superscripts and formulas
  sups:     Vec<Node>,
  hand:     Option<(Node, usize)>,
  /// the author reference marks' numbers its superscripts print (`_refmark`, `relocate::sup_marks`)
  refmarks: Vec<String>,
}

impl MarkGroup {
  /// Takes the group out of the name, `parents` the wrappers it leaves.
  fn cut(&self, document: &mut Document, parents: &mut Vec<Node>) {
    if let Some((text, at)) = &self.hand {
      let content = text.get_content();
      let mut text = text.clone();
      let _ = text.set_content(content[..*at].trim_end());
    }
    for sup in &self.sups {
      parents.extend(sup.get_parent());
      document.remove_node(sup.clone());
    }
  }
}

/// A piece of a name's ink, in print order ([`mark_groups`]).
enum Ink {
  Text(Node),
  /// a superscript or a formula of scripts alone, and its printed text
  Mark(Node, String),
  Other,
}

/// The mark runs of `personname` ([`MarkGroup`]): its superscripts and formulas of scripts alone outside notes and
/// links, but those that are
/// the name's ink — a lowercase one glued in a word (`James M<sup>c</sup>Kernan`, math0610203), a number's suffix
/// (`1<sup>st</sup> Siqiao Xue`, 2211.11758), a number glued before a word inside the name (`The <sup>3</sup>He
/// Collaboration`) — and the footnote symbols ending its last text ([`HAND_MARKS`]).
fn mark_groups(personname: &Node) -> Vec<MarkGroup> {
  fn walk(node: &Node, out: &mut Vec<Ink>) {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::TextNode) => out.push(Ink::Text(child)),
        Some(NodeType::ElementNode) => match child.get_name().as_str() {
          "sup" => {
            let printed = child.get_content().trim().to_string();
            // (a mark whose text a reference prints, resolved only after the conversion — cmpj's `\refaddr{la}`,
            // `<sup><ref labelref="LABEL:la"/></sup>` — has no text to give a note's `mark` yet: it stays)
            if name_ink(&child, &printed) || prints_later(&child) {
              out.push(Ink::Other);
            } else {
              out.push(Ink::Mark(child, printed));
            }
          },
          // (a formula of scripts alone is a mark, as the rewrite of math into text sets it after: `${}^{1}$`; any other
          // formula, and the scripts on a base, `$\text{Leon Bello}^{1,3}$`, the name's)
          "Math" => match fm_read_formula(&child) {
            FmFormulaMarks::Marks(scripts) => {
              let printed = scripts.join(",");
              if name_ink(&child, &printed) || prints_later(&child) {
                out.push(Ink::Other);
              } else {
                out.push(Ink::Mark(child, printed));
              }
            },
            _ => out.push(Ink::Other),
          },
          // (a link around the name holds its marks too, `\href{…}{Gert Desmet$^2$\includegraphics{orcid}}`, 2208.09944;
          // an ORCID iD's icon none)
          "ref"
            if !child
              .get_attribute("class")
              .is_some_and(|c| c.contains("ltx_orcid")) =>
          {
            walk(&child, out)
          },
          "note" | "ref" | "ERROR" | "graphics" | "picture" | "svg" => out.push(Ink::Other),
          _ => walk(&child, out),
        },
        _ => {},
      }
    }
  }
  let mut ink = Vec::new();
  walk(personname, &mut ink);
  // (the footnote symbols ending the last text: past them, the name's letters)
  let mut hand: Option<(usize, usize)> = None;
  if let Some(index) = ink
    .iter()
    .rposition(|piece| matches!(piece, Ink::Text(text) if !text.get_content().trim().is_empty()))
    && let Ink::Text(text) = &ink[index]
  {
    let content = text.get_content();
    let end = content.trim_end();
    let start = end
      .char_indices()
      .rev()
      .take_while(|(_, c)| HAND_MARKS.contains(c) || *c == ',' || c.is_whitespace())
      .last()
      .map_or(end.len(), |(at, _)| at);
    let run = &end[start..];
    let letters_before = content[..start].chars().any(char::is_alphabetic)
      || ink[..index].iter().any(
        |piece| matches!(piece, Ink::Text(t) if t.get_content().chars().any(char::is_alphabetic)),
      );
    if run.chars().any(|c| HAND_MARKS.contains(&c)) && letters_before {
      let start = start + (run.len() - run.trim_start_matches([',', ' ']).len());
      hand = Some((index, start));
    }
  }
  // (a mark with the name's ink on both sides parts a personname holding more than a name — an author block written whole,
  // `Ann Able, $^1$Bob Baker, … $^1$Univ A $^2$Univ B`, or names merged, `Ann Able$^*$ Bob Baker`, each counted by its
  // Warning or Error: its marks label what they stand between, and stay; ne2d, 108a)
  let inked = |piece: &Ink| matches!(piece, Ink::Text(text) if text.get_content().chars().any(char::is_alphanumeric));
  let mid = ink.iter().enumerate().any(|(index, piece)| {
    matches!(piece, Ink::Mark(..))
      && ink[..index].iter().any(inked)
      && ink[index + 1..].iter().any(inked)
  });
  if mid {
    return Vec::new();
  }
  let mut groups: Vec<MarkGroup> = Vec::new();
  let mut open = false;
  for (index, piece) in ink.iter().enumerate() {
    match piece {
      Ink::Mark(sup, content) => {
        if !open {
          groups.push(MarkGroup {
            printed:  String::new(),
            sups:     Vec::new(),
            hand:     None,
            refmarks: Vec::new(),
          });
          open = true;
        }
        let group = groups.last_mut().expect("an open group");
        // (two superscripts are two marks, `$^1$ $^2$` "1,2", 1304.0778; a comma set as one of its own is theirs,
        // `\authorrefmark{1}$^,$\authorrefmark{2}`, 2011.00594)
        if !group.printed.is_empty() && !group.printed.ends_with(',') && !content.starts_with(',') {
          group.printed.push(',');
        }
        group.printed.push_str(content);
        group.sups.push(sup.clone());
        if let Some(keys) = sup.get_attribute("_refmark") {
          group.refmarks.extend(keys.split(',').map(mark_form));
        }
      },
      Ink::Text(text) => match hand {
        Some((at, cut)) if at == index => {
          let content = text.get_content();
          groups.push(MarkGroup {
            printed:  content[cut..].trim_end().to_string(),
            sups:     Vec::new(),
            hand:     Some((text.clone(), cut)),
            refmarks: Vec::new(),
          });
          open = true;
        },
        _ => open &= text.get_content().trim().is_empty(),
      },
      Ink::Other => open = false,
    }
  }
  groups.retain(|group| !group.printed.trim().is_empty());
  groups
}

/// Whether `node` holds ink filled in after the conversion: a cross-reference or a citation of no text yet, whose text
/// the post-processor writes.
fn prints_later(node: &Node) -> bool {
  node.get_child_nodes().iter().any(|child| {
    child.get_type() == Some(NodeType::ElementNode)
      && ((matches!(child.get_name().as_str(), "ref" | "cite" | "bibref")
        && child.get_content().trim().is_empty())
        || prints_later(child))
  })
}

/// Whether a superscript (or a formula of scripts alone) printing `content` is the name's own ink, no mark
/// ([`mark_groups`]).
fn name_ink(sup: &Node, content: &str) -> bool {
  let before = sibling_text(sup, false);
  let after = sibling_text(sup, true);
  if glued_in_a_word(content, before.as_deref(), after.as_deref()) {
    return true;
  }
  let after_digit = before
    .as_deref()
    .and_then(|text| text.chars().next_back())
    .is_some_and(|c| c.is_ascii_digit());
  if after_digit && matches!(content, "st" | "nd" | "rd" | "th") {
    return true;
  }
  // (a number glued before a word, after words of the name: an isotope's prescript)
  !content.is_empty()
    && content.chars().all(|c| c.is_ascii_digit())
    && after
      .as_deref()
      .and_then(|text| text.chars().next())
      .is_some_and(char::is_uppercase)
    && before.as_deref().is_some_and(|text| {
      text.ends_with(char::is_whitespace) && text.chars().any(char::is_alphabetic)
    })
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

/// The copy of a mark a note's text opens with: its leading superscript or formula, or the head of its first text.
enum Lead {
  Sup(Node),
  Text(Node, usize),
}

impl Lead {
  fn cut(self, document: &mut Document) {
    match self {
      Lead::Sup(sup) => {
        // (and the space it printed before the text, `$^\dagger$ Also with …`)
        if let Some(mut text) = sup
          .get_next_sibling()
          .filter(|next| next.get_type() == Some(NodeType::TextNode))
        {
          let content = text.get_content();
          let _ = text.set_content(content.trim_start());
        }
        document.remove_node(sup);
      },
      Lead::Text(mut text, at) => {
        let content = text.get_content();
        let _ = text.set_content(content[at..].trim_start());
      },
    }
  }
}

/// The copy of `piece` `note`'s printed text opens with ([`Lead`]): a superscript or a formula of scripts alone printing
/// it, or, for a symbol, the head of its text where the mark ends there (`*`, not `**`) — a letter or a number in the
/// text is the note's own word (`\thanks{a visitor …}` under `Ann Able$^{a}$`).
fn leading_mark(note: &Node, piece: &str) -> Option<Lead> {
  // (its first printed text node, or the formula it opens with)
  fn first_ink(node: &Node) -> Option<Node> {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::TextNode) if !child.get_content().trim().is_empty() => return Some(child),
        Some(NodeType::ElementNode) if child.get_name() == "Math" => return Some(child),
        Some(NodeType::ElementNode) if child.get_name() != "tags" => {
          if let Some(ink) = first_ink(&child) {
            return Some(ink);
          }
        },
        _ => {},
      }
    }
    None
  }
  let text = first_ink(note)?;
  let key = mark_form(piece);
  if text.get_type() == Some(NodeType::ElementNode) {
    // (a formula of scripts alone printing the mark: `\thanks{$^{*}$Equal contribution}`)
    return match fm_read_formula(&text) {
      FmFormulaMarks::Marks(scripts) if mark_form(&scripts.join(",")) == key => {
        Some(Lead::Sup(text))
      },
      _ => None,
    };
  }
  // (inside a superscript of the note: that superscript, when it prints the mark alone)
  let mut up = text.get_parent();
  while let Some(parent) = up {
    if parent == *note {
      break;
    }
    if parent.get_name() == "sup" {
      return (mark_form(&parent.get_content()) == key).then_some(Lead::Sup(parent));
    }
    up = parent.get_parent();
  }
  if piece.chars().any(char::is_alphanumeric) {
    return None;
  }
  let content = text.get_content();
  let head = content.trim_start();
  let skipped = content.len() - head.len();
  let glyph = head
    .char_indices()
    .find(|(at, _)| mark_form(&head[..*at]) == key && *at > 0)
    .map(|(at, _)| at)
    .or_else(|| (mark_form(head) == key).then_some(head.len()))?;
  // (and not the head of a longer mark: `**`)
  let next = head[glyph..].chars().next();
  let continues = piece.chars().next_back().is_none_or(|last| {
    next.is_some_and(|c| mark_form(&c.to_string()) == mark_form(&last.to_string()))
  });
  if continues {
    return None;
  }
  Some(Lead::Text(text, skipped + glyph))
}

/// A mark-only note for `mark`, in `personname`'s namespace (Perl's `\footnotemark` note: `role="footnotemark"`),
/// flagged `_namemark`: no `\footnotetext` of the body is its text (`relocate_footnote`).
fn mark_only_note(document: &Document, personname: &Node, mark: &str) -> Result<Node> {
  let mut note = Node::new("note", None, document.get_document())?;
  if let Some(namespace) = personname.get_namespace() {
    let _ = note.set_namespace(&namespace);
  }
  note.set_attribute("role", "footnotemark")?;
  note.set_attribute("mark", mark)?;
  note.set_attribute("_namemark", "true")?;
  Ok(note)
}

/// The spaces a mark left at the name's edges, cut (`Conor Lawless <sup>*</sup>`, 2311.11099; `<sup>a,b</sup> Wolfgang
/// Wieland`, 1912.09514; the line end between `Lorenzo Leal$^1$` and its `$^2$`, 1304.0778), and the zero-width spaces set
/// between marks (`\up{\!1,2}`, 2209.14855).
fn trim_name_edges(document: &mut Document, personname: &Node) {
  fn blank(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{200B}' | '\u{2060}' | '\u{FEFF}')
  }
  fn edge_text(document: &mut Document, node: &Node, last: bool) -> Option<Node> {
    let mut children = node.get_child_nodes();
    if last {
      children.reverse();
    }
    for child in children {
      match child.get_type() {
        Some(NodeType::TextNode) if child.get_content().trim_matches(blank).is_empty() => {
          document.remove_node(child)
        },
        Some(NodeType::TextNode) => return Some(child),
        Some(NodeType::ElementNode) if child.get_name() == "text" => {
          return edge_text(document, &child, last);
        },
        _ => return None,
      }
    }
    None
  }
  if let Some(mut text) = edge_text(document, personname, true) {
    let content = text.get_content();
    if content.trim_end_matches(blank).len() != content.len() {
      let _ = text.set_content(content.trim_end_matches(blank));
    }
  }
  if let Some(mut text) = edge_text(document, personname, false) {
    let content = text.get_content();
    if content.trim_start_matches(blank).len() != content.len() {
      let _ = text.set_content(content.trim_start_matches(blank));
    }
  }
}
