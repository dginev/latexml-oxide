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

/// The attribute of a creator's contact that is part of a class's front-matter footnote, which `number_footnote_parts`
/// numbers: `title`, `own`, `join` (the name's joined footnote), `affil:<n>` or [`GIVEN_PART`].
pub const FOOTNOTE_PART: &str = "_fnpart";
/// The [`FOOTNOTE_PART`] of a name's `\footnote` given its number (`\footnote[1]{…}`), which steps no counter.
pub const GIVEN_PART: &str = "given";
/// How a class that writes [`FOOTNOTE_PART`]s marks its front-matter footnotes: `fnsymbol` or `alph`.
pub const FOOTNOTE_MARKS: &str = "lx_fm_footnote_marks";

/// The marks of a class's front-matter footnotes (REVTeX's `\@AF@join`, revtex4-2.cls:2404-2495), numbered as printed:
/// the title's, then each name's own and its joined one, an affiliation's after its group's names or after every name, a
/// text printed before keeping its number (`\y@match`). A `\footnote` in a name steps the counter where the name prints
/// and keeps its contact, unmarked; a `\thanks` there, which REVTeX prints nowhere, takes no number and no mark. A mark
/// only where both affiliation orders agree; none when a footnote's step is unknown — a footnote contact with no part, a
/// name's `\footnotemark`, a part outside the creators (OD #1054).
pub(super) fn number_footnote_parts(document: &mut Document) -> Result<()> {
  let style = lookup_string(FOOTNOTE_MARKS);
  if style.is_empty() {
    return Ok(());
  }
  let creators = document.findnodes("//ltx:creator", None);
  // (each part: its node, its creator, its part, its text — an own part's as printed, a joined one's with its role)
  let mut parts: Vec<(Node, usize, String, String)> = Vec::new();
  // (a part a route placed outside the creators — a title's or a pubnote's — is a step not counted here)
  let mut unknown = !document
    .findnodes(
      &s!(
        "//*[@{FOOTNOTE_PART} and @{FOOTNOTE_PART}!='{GIVEN_PART}' and not(ancestor::ltx:creator)]"
      ),
      None,
    )
    .is_empty();
  for (at, creator) in creators.iter().enumerate() {
    let entries = document.findnodes(
      "ltx:contact | ltx:note[@role='footnotemark']",
      Some(creator),
    );
    for entry in entries {
      let role = entry.get_attribute("role").unwrap_or_default();
      let text = entry
        .get_content()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
      match (entry.get_attribute(FOOTNOTE_PART), role.as_str()) {
        // (a given number, `\footnote[1]{…}`, steps nothing: 1607.07280)
        (Some(part), _) if part == GIVEN_PART => {},
        (Some(part), _) if part == "own" => parts.push((entry, at, part, text)),
        (Some(part), _) => {
          let name = entry.get_attribute("name").unwrap_or_default();
          parts.push((entry, at, part, s!("{role}\u{2}{name}\u{2}{text}")));
        },
        // (a name's `\footnote`, its own footnote where the name prints, never shared: "Ann Able∗ / Bob Baker† Cy Cole†"
        // in REVTeX 4.0, "Ann Ablea" in 4.2's minipage letters)
        (None, "note") => {
          let own = s!("\u{3}{}", parts.len());
          parts.push((entry, at, s!("own"), own));
        },
        // (a name's `\footnotemark` steps the counter unless given a number, which its note no longer tells)
        (None, "footnotemark") => unknown = true,
        (None, _) => {
          unknown |= matches!(role.as_str(), "thanks" | "email" | "url" | "altaffiliation")
        },
      }
    }
  }
  for (entry, ..) in &mut parts {
    entry.remove_attribute(FOOTNOTE_PART).ok();
  }
  if unknown || parts.is_empty() {
    return Ok(());
  }
  // (each part's footnote: its own text, or its parts' joined in order — the creator's, the title's, the affiliation's)
  let group = |at: usize, part: &str| match part {
    "title" => s!("title"),
    _ if part.starts_with("affil:") => part.to_string(),
    _ => s!("join:{at}"),
  };
  let mut joined: HashMap<String, String> = HashMap::default();
  for (_, at, part, text) in parts.iter().filter(|(_, _, part, _)| part != "own") {
    let footnote = joined.entry(group(*at, part)).or_default();
    footnote.push_str(text);
    footnote.push('\u{1}');
  }
  let footnotes: Vec<String> = parts
    .iter()
    .map(|(_, at, part, text)| match part.as_str() {
      "own" => text.clone(),
      _ => joined[&group(*at, part)].clone(),
    })
    .collect();
  // (the printed order: the title's, each name's own then its joined one, an affiliation's after its group's names —
  // its footnote on the group's last name, where `\email` after `\affiliation` puts it — or after every name)
  let numbers = |affiliations_last: bool| -> HashMap<&String, usize> {
    let mut order: Vec<&String> = Vec::new();
    fn add<'a>(footnote: &'a String, order: &mut Vec<&'a String>) {
      if !order.contains(&footnote) {
        order.push(footnote);
      }
    }
    let of = |which: &dyn Fn(usize, &str) -> bool| -> Vec<&String> {
      parts
        .iter()
        .zip(&footnotes)
        .filter(|((_, at, part, _), _)| which(*at, part))
        .map(|(_, footnote)| footnote)
        .collect()
    };
    for footnote in of(&|_, part| part == "title") {
      add(footnote, &mut order);
    }
    let mut affiliations: Vec<&String> = Vec::new();
    for holder in 0..creators.len() {
      for footnote in of(&|at, part| at == holder && part == "own") {
        add(footnote, &mut order);
      }
      for footnote in of(&|at, part| at == holder && part == "join") {
        add(footnote, &mut order);
      }
      for footnote in of(&|at, part| at == holder && part.starts_with("affil:")) {
        if affiliations_last {
          add(footnote, &mut affiliations);
        } else {
          add(footnote, &mut order);
        }
      }
    }
    for footnote in affiliations {
      add(footnote, &mut order);
    }
    order
      .into_iter()
      .enumerate()
      .map(|(n, footnote)| (footnote, n + 1))
      .collect()
  };
  let (grouped, last) = (numbers(false), numbers(true));
  // (a document's or package's own `\@fnsymbol` prints glyphs not this table's, 2603.18989's †: none written, the mark
  // unknown, `insert::count_unknown_marks`)
  let own_fnsymbol = style != "alph"
    && lookup_definition(&T_CS!("\\@fnsymbol"))
      .ok()
      .flatten()
      .is_some_and(|d| !d.get_origin().is_latexml_owned());
  for ((entry, ..), footnote) in parts.iter().zip(&footnotes) {
    let mut entry = entry.clone();
    let n = grouped[footnote];
    if entry.get_attribute("role").as_deref() != Some("thanks") || n != last[footnote] {
      continue;
    }
    let mark = match style.as_str() {
      "alph" => u8::try_from(n)
        .ok()
        .filter(|n| (1..=26).contains(n))
        .map(|n| char::from(b'a' + n - 1).to_string()),
      _ => ["∗", "†", "‡", "§", "¶", "‖", "∗∗", "††", "‡‡"]
        .get(n - 1)
        .map(|mark| (*mark).to_string()),
    };
    if own_fnsymbol {
      if entry.get_name() == "note" {
        entry.set_attribute(MARK_UNKNOWN, "true")?;
      }
      continue;
    }
    // (a thanks note its mark, a contact the mark the Kind step reads)
    if let Some(mark) = mark {
      if entry.get_name() == "note" {
        entry.set_attribute("mark", &mark)?;
        entry.set_attribute(MARK_FROM_COUNTER, "true")?;
      } else {
        entry.set_attribute(PRINTED_MARK, &mark)?;
      }
    }
  }
  Ok(())
}

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
    note.set_attribute(MARK_FROM_COUNTER, "true")?;
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
            // (a footnote symbol set in math, not raised: cnf.sty's `$\ast$~…`)
            _ => Some(child.get_content().trim().to_string())
              .filter(|text| !text.is_empty() && text.chars().all(events::is_fnsymbol))
              .map(|text| (child.clone(), text)),
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
/// mark a legend answers does (`settle_name_marks`). A name's superscript of other marks too (`$^{1,\ddagger}$`)
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
