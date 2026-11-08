//! A list of e-mail addresses one contact prints for several people, parted into each address's e-mail on its owner
//! (census138 ADDR-b; OXIDIZED_DESIGN_DIVERGENCES #972).

use super::*;

/// One address of a contact's list ([`listed_addresses`]): the address, the name printed beside it (`a@x (Ann Able)`,
/// `Ann Able (a@x)`), if any, the bytes of the contact's text it takes with that name and its brackets, and those it
/// takes alone.
struct ListedAddress {
  address: String,
  name:    Option<String>,
  span:    std::ops::Range<usize>,
  bare:    std::ops::Range<usize>,
}

/// FIDELITY_CHARTER §4, ownership by evidence: a contact printing a list of addresses for several people — a shared
/// corresponding-author note copied whole to each of its carriers (elsarticle's `\cortext[cor1]{Corresponding
/// authors:\\ zjsong@…; Chenxizhang@…}`, 2302.03296; REVTeX's `\thanks{Corresponding Authors (chunfeng.lian@…;
/// wzy218@…; cong6406@…; jhma@…)}` on four authors, 2508.07950), or an e-mail call naming the owner beside each address
/// (`\email[Corresponding author]{: xiazh@… (Zhenhua Xia); lianghongstefanie@… (Hong Liang)}` on Hong Liang's call,
/// 2009.14655; `\cortext{… Abrar Faiyaz (abrar\_faiyaz@…) and Md Nasir Uddin (nasir\_uddin@…)}`, 2507.07920) — gives each
/// address as an e-mail to its owner, by the placement order: the author whose name is printed beside it, the strongest
/// evidence; else the author it spells; else, among the contact's carriers, the consistent order or the elimination
/// (`owners::list_owners`); else the block's own creator (OD #159), once. The names printed beside the addresses
/// are no e-mail's text. The contact's other nodes stay its carriers', as printed — only the address spans and the
/// separators beside them are cut out of it, in place ([`cut_ranges`], [`cut_printed`]): its words (`Corresponding
/// authors:`; an "and"/"or" outside the list), links, breaks and math (2311.15476's DOI link, review rev107f B2); a
/// contact left with no word is dropped. An e-mail keeps the call's name (`Corresponding author`, which `write_role_notes` then makes each
/// owner's correspondence note); a note's addresses take the e-mail's own. Read once every route has placed the
/// contacts, before the correspondence labels (census138 ADDR-b, brief 107f). A contact one creator holds is parted only
/// where every address has an author's name beside it; an e-mail whose list prints other words, or a list none of whose
/// addresses finds an author, is left as it is.
pub(super) fn distribute_address_lists(document: &mut Document) -> Result<()> {
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  if authors.len() < 2 {
    return Ok(());
  }
  let names: Vec<String> = authors
    .iter()
    .map(|author| {
      document
        .findnode("ltx:personname", Some(author))
        .map(|person| person.get_content())
        .unwrap_or_default()
    })
    .collect();
  // (a name not yet written leaves the evidence incomplete: the lists stay as printed)
  if names.iter().any(|name| name.trim().is_empty()) {
    return Ok(());
  }
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| NameKeys::of(&Tokens::new(ExplodeText!(name))))
    .collect();
  let printed_names: Vec<String> = names.iter().map(|name| name_letters(name)).collect();
  // (each printed contact and its holders: one role, name and text)
  type Group = (Option<String>, Option<String>, String, Vec<(usize, Node)>);
  let mut groups: Vec<Group> = Vec::new();
  for (at, author) in authors.iter().enumerate() {
    for contact in document.findnodes(
      "ltx:contact[@role='email' or @role='correspondent' or @role='thanks']",
      Some(author),
    ) {
      let text = printed_text(&contact);
      let (role, name) = (contact.get_attribute("role"), contact.get_attribute("name"));
      match groups
        .iter_mut()
        .find(|(r, n, t, _)| *r == role && *n == name && *t == text)
      {
        Some((.., holders)) => holders.push((at, contact)),
        None => groups.push((role, name, text, vec![(at, contact)])),
      }
    }
  }
  let default_name = get_frontmatter_name(None, "ltx:contact", "email")?;
  for (role, name, text, holders) in groups {
    let listed = listed_addresses(&text);
    if listed.len() < 2 {
      continue;
    }
    let is_email = role.as_deref() == Some("email");
    let printed_owner = |item: &ListedAddress| {
      let name = name_letters(item.name.as_deref()?);
      let mut owners =
        (0..printed_names.len()).filter(|&at| !name.is_empty() && printed_names[at] == name);
      match (owners.next(), owners.next()) {
        (Some(owner), None) => Some(owner),
        _ => None,
      }
    };
    let mut carriers: Vec<usize> = holders.iter().map(|(at, _)| *at).collect();
    carriers.dedup();
    if carriers.len() < 2 && !listed.iter().all(|item| printed_owner(item).is_some()) {
      continue;
    }
    // (a name beside an address is cut with it only where it is an author's: a remark stays, `\{ann,bob\}@a.org
    // (corresponding)` — perfect_kernel_batch92c)
    let spans: Vec<std::ops::Range<usize>> = listed
      .iter()
      .map(|item| {
        if printed_owner(item).is_some() {
          item.span.clone()
        } else {
          item.bare.clone()
        }
      })
      .collect();
    let cuts = cut_ranges(&text, &spans, !is_email);
    let rest = without_ranges(&text, &cuts);
    if is_email && rest.chars().any(char::is_alphanumeric) {
      continue;
    }
    // (the author printed beside it; else the one it spells; else, among the carriers, the order or the elimination)
    let mut owners: Vec<Option<usize>> = listed
      .iter()
      .map(|item| printed_owner(item).or_else(|| spelled_owner(&keys, &item.address)))
      .collect();
    if owners.iter().any(Option::is_none)
      && owners
        .iter()
        .flatten()
        .all(|owner| carriers.contains(owner))
    {
      let carrier_keys: Vec<NameKeys> = carriers.iter().map(|&at| keys[at].clone()).collect();
      let addresses: Vec<String> = listed.iter().map(|item| item.address.clone()).collect();
      let spelled: Vec<Option<usize>> = owners
        .iter()
        .map(|owner| owner.and_then(|owner| carriers.iter().position(|&c| c == owner)))
        .collect();
      let taken = vec![false; carriers.len()];
      for (at, (owner, _)) in list_owners(&carrier_keys, &addresses, &spelled, &taken, false, false)
        .into_iter()
        .enumerate()
      {
        if owners[at].is_none() {
          owners[at] = owner.map(|owner| carriers[owner]);
        }
      }
    }
    DebugFeature!(
      "frontmatter",
      "FRONT address list of a {role:?} on {:?}: {:?} owners {owners:?}",
      carriers
        .iter()
        .map(|&at| names[at].as_str())
        .collect::<Vec<_>>(),
      listed
        .iter()
        .map(|item| item.address.as_str())
        .collect::<Vec<_>>()
    );
    // (a list none of whose addresses finds an author names no one: it stays as printed — PoS's `\speaker{Rob
    // Fender$^{1,2*}$, Patrick Woudt$^2$}` and its `E-mail: \email{rob.fender@…, Patrick.Woudt@…}`, whose names are no
    // creators yet here, 1711.04132)
    if owners.iter().all(Option::is_none) {
      continue;
    }
    let email_name = if is_email {
      name.clone()
    } else {
      default_name.clone()
    };
    for (item, owner) in listed.iter().zip(&owners) {
      let target = match owner {
        Some(owner) => Some(authors[*owner].clone()),
        None => block_creator(document, &authors)?,
      };
      let Some(mut target) = target else {
        continue;
      };
      let held = document
        .findnodes("ltx:contact[@role='email']", Some(&target))
        .iter()
        .any(|contact| contact.get_content().trim() == item.address);
      if held {
        continue;
      }
      let mut contact = Node::new("contact", None, document.get_document())?;
      if let Some(namespace) = target.get_namespace() {
        let _ = contact.set_namespace(&namespace);
      }
      contact.set_attribute("role", "email").ok();
      if let Some(name) = &email_name {
        contact.set_attribute("name", name).ok();
      }
      contact.append_text(&item.address)?;
      // (after the list it printed in, where its owner holds it; else after the owner's contacts)
      match holders.iter().find(|(at, _)| Some(*at) == *owner) {
        Some((_, list)) => {
          list.clone().add_next_sibling(&mut contact)?;
        },
        None => {
          target.add_child(&mut contact)?;
        },
      }
    }
    // (the list's other nodes stay its carriers', as printed — its words, links, breaks and math (2311.15476's DOI
    // link, review rev107f B2); nothing left, the contact goes)
    for (_, list) in holders {
      if rest.chars().any(char::is_alphanumeric) {
        cut_printed(document, &list, &cuts);
      } else {
        document.remove_node(list);
      }
    }
  }
  Ok(())
}

/// A contact's text as printed, a line break (`ltx:break`) a line's end.
fn printed_text(node: &Node) -> String {
  let mut text = String::new();
  for child in node.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) {
      if child.get_name() == "break" {
        text.push('\n');
      } else {
        text.push_str(&printed_text(&child));
      }
    } else {
      text.push_str(&child.get_content());
    }
  }
  text
}

/// A name's letters, lowercased, by their base letters: what a name printed beside an address is compared by.
fn name_letters(name: &str) -> String {
  resolver::base_letters(name)
    .chars()
    .filter(|c| c.is_alphabetic())
    .flat_map(char::to_lowercase)
    .collect()
}

/// The addresses a contact's text prints ([`ListedAddress`]): each plain address, a braced group of local parts
/// written out (`{a, b}@x`, `resolver::addresses_in`), with the name printed in brackets after it (`a@x (Ann Able)`) or
/// before its bracketed address (`Ann Able (a@x)`), back to a separator, a line's start or an "and".
fn listed_addresses(text: &str) -> Vec<ListedAddress> {
  let mut out: Vec<ListedAddress> = Vec::new();
  let mut from = 0;
  for (at, _) in text.match_indices('@') {
    if at < from {
      continue;
    }
    let domain_len = text[at + 1..]
      .find(|c: char| !(c.is_ascii_alphanumeric() || ".-".contains(c)))
      .unwrap_or(text.len() - at - 1);
    let domain = text[at + 1..at + 1 + domain_len].trim_end_matches(['.', '-']);
    if !domain.contains('.') {
      continue;
    }
    let end = at + 1 + domain.len();
    let (start, addresses) = if text[..at].ends_with('}') {
      let Some(open) = text[from..at].rfind('{').map(|open| from + open) else {
        continue;
      };
      (open, resolver::addresses_in(&text[open..end]))
    } else {
      let start = text[from..at]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_local_char(*c))
        .last()
        .map_or(at, |(i, _)| from + i);
      if start == at {
        continue;
      }
      (start, vec![text[start..end].to_string()])
    };
    from = end;
    if addresses.is_empty() {
      continue;
    }
    let before = text[..start].trim_end();
    let after = text[end..].trim_start();
    let (mut span, mut name) = (start..end, None);
    if before.ends_with('(') && after.starts_with(')') {
      // (`Ann Able (a@x)`: the name before the bracket, back to a separator, a line's start or an "and")
      let open = before.len() - 1;
      let head = &text[..open];
      let name_start = head
        .rfind([':', ';', ',', '\n', '(', ')'])
        .map_or(0, |p| p + 1);
      let mut words = head[name_start..].trim();
      let mut lead =
        name_start + (head[name_start..].len() - head[name_start..].trim_start().len());
      if let Some(rest) = words.strip_prefix("and ") {
        lead += words.len() - rest.len();
        words = rest.trim_start();
      }
      span = lead..text.len() - after.len() + 1;
      name = (!words.is_empty()).then(|| words.to_string());
    } else if let Some(tag) = after.strip_prefix('(')
      && let Some(close) = tag.find(')')
      && !tag[..close].contains('@')
    {
      // (`a@x (Ann Able)`)
      let tag_start = text.len() - after.len();
      span = start..tag_start + 1 + close + 1;
      name = Some(tag[..close].trim().to_string());
    }
    from = from.max(span.end);
    for address in addresses {
      out.push(ListedAddress {
        address,
        name: name.clone(),
        span: span.clone(),
        bare: start..end,
      });
    }
  }
  out
}

/// The bytes of a contact's printed `text` ([`printed_text`]) its listed addresses take ([`listed_addresses`]), which
/// [`cut_printed`] cuts out of the contact: their `spans`; between two of them, the separators — spaces, line breaks,
/// `;` `,` `&` and an "and" or "or", only there; and around each run of them, the space before it, the brackets it
/// leaves empty (`Corresponding Authors (a; b)` leaves "Corresponding Authors"), a `;` or `,` after it, a stop after it
/// where a label or nothing opens it (`Corresponding authors:\\ a; b.` leaves "Corresponding authors:"). `strip_label`:
/// an e-mail label opening the run after other words too, with that stop (`… addressed. Email: a or b.` leaves
/// `… addressed.`, 2307.11827; a correspondent's own `e-mail: a, b, c` keeps its one word, 2409.06261; an e-mail's own
/// label stays, so its list stays whole as printed: `Emails: \{ann,bob\}@a.org`, perfect_kernel_batch92c).
fn cut_ranges(
  text: &str,
  spans: &[std::ops::Range<usize>],
  strip_label: bool,
) -> Vec<std::ops::Range<usize>> {
  let mut spans = spans.to_vec();
  spans.sort_by_key(|span| (span.start, span.end));
  spans.dedup();
  // (the runs of addresses only separators part)
  let separators = |gap: &str| {
    gap
      .split(|c: char| c.is_whitespace() || ";,&".contains(c))
      .all(|word| word.is_empty() || word == "and" || word == "or")
  };
  let mut runs: Vec<(usize, usize)> = Vec::new();
  for span in spans {
    match runs.last_mut() {
      Some((_, end)) if span.start < *end => {},
      Some((_, end)) if separators(&text[*end..span.start]) => *end = span.end,
      _ => runs.push((span.start, span.end)),
    }
  }
  let space_after = |at: usize| at + (text[at..].len() - text[at..].trim_start().len());
  let space_before = |at: usize| text[..at].trim_end().len();
  let mut cuts: Vec<std::ops::Range<usize>> = Vec::new();
  for (start, end) in runs {
    let (mut lo, mut hi) = (space_before(start), end);
    let next = space_after(end);
    let opener = text[..lo].trim_end();
    if opener.ends_with('(') && text[next..].starts_with(')') {
      lo = space_before(lo - 1);
      hi = next + 1;
    } else if text[next..].starts_with([';', ','])
      || (text[next..].starts_with('.')
        && (opener.ends_with(':') || opener.trim_matches([';', ',', ':']).is_empty()))
    {
      hi = next + 1;
    }
    if strip_label {
      let head = text[..lo].trim_end();
      // (past the last space, whatever its width: a no-break space is two bytes)
      let word = head
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map_or(0, |(at, c)| at + c.len_utf8());
      let label = &head[word..];
      if label.ends_with(':')
        && is_email_label(label.trim_end_matches(':'))
        && text[..word].chars().any(char::is_alphanumeric)
      {
        lo = space_before(word);
      }
    }
    // (a run opening the text, a stray `:` or `;` before it aside, takes the space after it)
    if text[..lo]
      .trim_matches(|c: char| c.is_whitespace() || ";,:".contains(c))
      .is_empty()
    {
      lo = 0;
      hi = space_after(hi);
    }
    cuts.push(lo..hi);
  }
  cuts
}

/// `text` without the bytes of `cuts` ([`cut_ranges`]).
fn without_ranges(text: &str, cuts: &[std::ops::Range<usize>]) -> String {
  let mut kept = String::new();
  let mut at = 0;
  for cut in cuts {
    kept.push_str(&text[at..cut.start.max(at)]);
    at = at.max(cut.end);
  }
  kept.push_str(&text[at.min(text.len())..]);
  kept
}

/// Cuts the bytes `cuts` of a contact's printed text ([`printed_text`], [`cut_ranges`]) out of the contact, in place:
/// each text node keeps its other characters, a line break or an element all of whose printed text is cut goes, and
/// every other node stays as it is — the note's words, links, breaks and math (review rev107f B2: 2311.15476's
/// `Corresponding authors: … DOI: \href{…}`).
fn cut_printed(document: &mut Document, contact: &Node, cuts: &[std::ops::Range<usize>]) {
  // (a snapshot of the printed pieces, each with its bytes of the printed text: the text nodes, the breaks, the
  // elements around them)
  enum Piece {
    Text(Node, std::ops::Range<usize>),
    Element(Node, std::ops::Range<usize>),
  }
  fn walk(node: &Node, at: &mut usize, pieces: &mut Vec<Piece>) {
    for child in node.get_child_nodes() {
      if child.get_type() == Some(NodeType::ElementNode) {
        let start = *at;
        if child.get_name() == "break" {
          *at += 1;
        } else {
          walk(&child, at, pieces);
        }
        pieces.push(Piece::Element(child, start..*at));
      } else {
        let len = child.get_content().len();
        if matches!(
          child.get_type(),
          Some(NodeType::TextNode | NodeType::CDataSectionNode)
        ) {
          pieces.push(Piece::Text(child, *at..*at + len));
        }
        *at += len;
      }
    }
  }
  let mut pieces: Vec<Piece> = Vec::new();
  walk(contact, &mut 0, &mut pieces);
  let covered = |range: &std::ops::Range<usize>| {
    !range.is_empty()
      && cuts
        .iter()
        .any(|cut| cut.start <= range.start && range.end <= cut.end)
  };
  // (the outermost elements wholly cut go whole; the text inside them with them)
  let mut gone: Vec<std::ops::Range<usize>> = Vec::new();
  let inside = |gone: &[std::ops::Range<usize>], range: &std::ops::Range<usize>| {
    gone
      .iter()
      .any(|g| g.start <= range.start && range.end <= g.end && !range.is_empty())
  };
  // (outermost first: an element's piece follows its children's, so walk the snapshot from the end)
  for piece in pieces.iter().rev() {
    if let Piece::Element(node, range) = piece
      && covered(range)
      && !inside(&gone, range)
    {
      gone.push(range.clone());
      document.remove_node(node.clone());
    }
  }
  for piece in &pieces {
    let Piece::Text(node, range) = piece else {
      continue;
    };
    if inside(&gone, range) {
      continue;
    }
    let content = node.get_content();
    let local: Vec<std::ops::Range<usize>> = cuts
      .iter()
      .filter(|cut| cut.start < range.end && range.start < cut.end)
      .map(|cut| cut.start.max(range.start) - range.start..cut.end.min(range.end) - range.start)
      .collect();
    if local.is_empty() {
      continue;
    }
    let kept = without_ranges(&content, &local);
    let mut node = node.clone();
    if kept.is_empty() {
      node.unlink_node();
    } else {
      let _ = node.set_content(&kept);
    }
  }
}
