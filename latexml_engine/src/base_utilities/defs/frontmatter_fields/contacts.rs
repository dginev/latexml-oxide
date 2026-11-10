//! The contact fields' own readers: the owner of a contact placed apart from the names (`\lx@add@contact@placed`), and
//! the text of an address as its field writes it. Moved unchanged from `author_block/affiliations.rs` and
//! `author_block/email_owners.rs` (Z18a): field logic, not author-block parsing.

use super::*;

/// Where a contact placed apart from the names goes ([`placed_contact_owner`]).
pub(in crate::base_utilities) enum ContactOwner {
  /// to the authors requesting the mark's label, which pairs them as an affiliation's mark does (and so the mark,
  /// answered, is not left standing on the name)
  Mark(String),
  /// to the author at this queue position (from 1)
  Author(usize),
  /// to the whole block, the shared creator below the authors (#159)
  Block,
}

/// The owner of a contact placed apart from the names (`\lx@add@contact@placed`), in this order:
/// 1. the mark written on the contact itself (`$^*$x@y`), when one queued author requests it — the contact's own mark
///    is the strongest evidence, stronger than the spelling (`*b@x` under `A$^*$` and `B` is A's); when several
///    request it, the one of them the address spells (1312.7046);
/// 2. the author whose surname the address's local part spells ([`spelled_owners`]: 0901.2914);
/// 3. the only author of the current group (`\lx@frontmatter@group@start`; the whole queue when no class marks
///    groups);
/// 4. else the block's.
///
/// An author block's labelled email lines rank otherwise (`block_email_calls`): there the spelling outranks the mark
/// of a correspondence NOTE before the line (1710.00802's `aleder@` under another author's "Corresponding Author"),
/// a mark on a nearby note being weaker evidence than one on the address.
///
/// A `weak` mark is not written on the contact but on the address block the class prints it under (iopart's `\ead`
/// after an `\address`, `\lx@address@mark@set`): it places only an address that spells no one (1111.5264's `johnwc@`
/// under `$^6$`, which two authors request, is J W Conklin's `$^1$`; 2407.14305's `rockfish3141@` under `$^1$ Waseda`,
/// Hidetomo Hoshino's alone).
pub(in crate::base_utilities) fn placed_contact_owner(
  mark: &Tokens,
  content: &Tokens,
  weak: bool,
) -> ContactOwner {
  let authors = queued_author_marks();
  let keys = queued_author_name_keys();
  let spelled: Vec<usize> = address_owners(&keys, content)
    .into_iter()
    .map(|at| at + 1)
    .collect();
  let requesting = if mark.is_empty() || (weak && !spelled.is_empty()) {
    Vec::new()
  } else {
    queued_requesters(mark.unlist_ref())
  };
  if let [author] = requesting.as_slice() {
    // (a footnote-symbol mark stays shown on the name and pairs by no label of its own: the author's position)
    return match mark_label(mark.unlist_ref()) {
      Some(label) if !is_symbol_mark(mark.unlist_ref()) => ContactOwner::Mark(label),
      _ => ContactOwner::Author(*author),
    };
  }
  if let [author] = requesting
    .iter()
    .copied()
    .filter(|author| spelled.contains(author))
    .collect::<Vec<_>>()
    .as_slice()
  {
    return ContactOwner::Author(*author);
  }
  if let [author] = spelled.as_slice() {
    return ContactOwner::Author(*author);
  }
  let first = frontmatter_group_first();
  if authors.len() == first {
    return ContactOwner::Author(first);
  }
  ContactOwner::Block
}

/// `text` as tokens of its characters, each as it is: an ASCII letter a letter, anything else other (no `#` parameter, no
/// `%` comment, no `{`/`}` group: a keyval value that is text).
pub(in crate::base_utilities) fn text_tokens(text: &str) -> Vec<Token> {
  text
    .chars()
    .map(|c| {
      let c = c.to_string();
      if c.chars().all(|c| c.is_ascii_alphabetic()) {
        T_LETTER!(&c)
      } else if c == " " {
        T_SPACE!()
      } else {
        T_OTHER!(&c)
      }
    })
    .collect()
}

/// The key a footnote-symbol mark (`$^\dagger$` on `$^\dagger$baker@y.edu`) answers on the author it places a contact
/// for ([`symbol_mark_key`]). `None` for any other mark.
pub(in crate::base_utilities) fn answered_symbol_key(mark: &[Token]) -> Option<String> {
  if mark.is_empty() || !is_symbol_mark(mark) {
    return None;
  }
  let label = mark_label(mark)?;
  symbol_mark_key(label.strip_prefix("affiliation:").unwrap_or(&label))
}

/// The queue position of the current author group's first author (`\lx@frontmatter@group@start`; 1 when no class marks
/// groups).
pub(in crate::base_utilities) fn frontmatter_group_first() -> usize {
  match lookup_value("lx_frontmatter_group_first") {
    Some(Stored::Number(n)) => usize::try_from(n.value_of()).unwrap_or(1).max(1),
    _ => 1,
  }
}

/// An address's tokens as TeX prints them where a brace is set in math: `$\{$`, `$\lbrace$` the brace `\{`, `$\}$`,
/// `$\rbrace$` the brace `\}` — the math shifts read as characters (a class's verbatim `\email` argument: llncs's
/// `\email{$\{$fargyriou,mikebekos,symvonis$\}$@math.ntua.gr}`, 1106.2694, 2308.00146, 1602.04934) or as shifts —, so
/// a list of local parts in math braces is the list in text braces (`\lx@add@email`, `\lx@split@addresses`; census138
/// ADDR-b, OXIDIZED_DESIGN_DIVERGENCES #972). The shifts pair left to right, as TeX pairs them; any other formula
/// stays as it is.
pub(in crate::base_utilities) fn math_braces_as_printed(address: &Tokens) -> Tokens {
  let v = address.unlist_ref();
  let shift = |t: &Token| *t == T_OTHER!("$") || t.get_catcode() == Catcode::MATH;
  let brace = |t: &Token| {
    if *t == T_CS!("\\{") || *t == T_CS!("\\lbrace") {
      Some(T_CS!("\\{"))
    } else if *t == T_CS!("\\}") || *t == T_CS!("\\rbrace") {
      Some(T_CS!("\\}"))
    } else {
      None
    }
  };
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut at = 0;
  while at < v.len() {
    // the shifts pair left to right, as TeX pairs them: any other pair stays as read, whole, and the next pair opens
    // after its closing shift (`$x$\}$y$` keeps both formulas)
    if shift(&v[at])
      && let Some(len) = v[at + 1..].iter().position(shift)
    {
      let pair = &v[at..at + len + 2];
      let mut inner = pair[1..=len].iter().filter(|t| **t != T_SPACE!());
      match (inner.next().and_then(brace), inner.next()) {
        (Some(printed), None) => out.push(printed),
        _ => out.extend_from_slice(pair),
      }
      at += len + 2;
      continue;
    }
    out.push(v[at]);
    at += 1;
  }
  Tokens::new(out)
}
