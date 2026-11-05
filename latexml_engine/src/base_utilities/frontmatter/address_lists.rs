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
/// (`email_owners::list_owners`); else the block's own creator (OD #159), once. The names printed beside the addresses
/// are no e-mail's text. The contact's other words stay its carriers' (`Corresponding authors:`); a contact left with
/// none is dropped. An e-mail keeps the call's name (`Corresponding author`, which `write_role_notes` then makes each
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
    let rest = remainder(&text, &spans, !is_email);
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
    // (the list's other words stay its carriers'; nothing left, the contact goes)
    for (_, mut list) in holders {
      if rest.chars().any(char::is_alphanumeric) {
        for mut child in list.get_child_nodes() {
          child.unlink_node();
        }
        list.append_text(&rest)?;
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

/// The words of `text` its listed addresses leave ([`listed_addresses`]): their `spans` cut, the separators and the
/// "and" between them and the brackets they leave empty dropped, a line's end a space (`Corresponding Authors (a; b)`
/// leaves "Corresponding Authors", `Corresponding authors:\n a; b` "Corresponding authors:"); `strip_label`, an e-mail
/// label left with nothing after it dropped too.
fn remainder(text: &str, spans: &[std::ops::Range<usize>], strip_label: bool) -> String {
  let mut kept = String::new();
  let mut at = 0;
  let mut spans = spans.to_vec();
  spans.dedup();
  for span in spans {
    if span.start >= at {
      kept.push_str(&text[at..span.start]);
      kept.push(' ');
      at = span.end;
    }
  }
  kept.push_str(&text[at.min(text.len())..]);
  let words: Vec<&str> = kept
    .split_whitespace()
    .filter(|word| !matches!(*word, ";" | "," | "and" | "&" | "&amp;" | "or"))
    .collect();
  let mut rest = words.join(" ");
  loop {
    let before = rest.clone();
    for empty in ["()", "( )", "[]", "[ ]"] {
      rest = rest.replace(empty, "");
    }
    rest = rest
      .trim()
      .trim_end_matches([';', ',', '&'])
      .trim_start_matches([';', ',', ':'])
      .trim()
      .to_string();
    // (a note's e-mail label its addresses left, and a stop after it: `… addressed. Email: or .` leaves `… addressed.`,
    // 2307.11827 — `strip_label`; an e-mail's own label stays, so its list stays whole as printed: `Emails:
    // \{ann,bob\}@a.org`, perfect_kernel_batch92c)
    if strip_label
      && let Some(head) = rest.strip_suffix('.').map(str::trim_end)
      && head.ends_with(':')
    {
      rest = head.to_string();
    }
    // (only after other words: a correspondent's `e-mail: a, b, c` keeps its label, the annotation's one word —
    // 2409.06261)
    let (head, label) = rest.rsplit_once(' ').unwrap_or(("", rest.as_str()));
    if strip_label
      && head.chars().any(char::is_alphanumeric)
      && label.ends_with(':')
      && is_email_label(label.trim_end_matches(':'))
    {
      let head = head.trim_end().to_string();
      rest = head;
    }
    if rest == before {
      break;
    }
  }
  rest.split_whitespace().collect::<Vec<_>>().join(" ")
}
