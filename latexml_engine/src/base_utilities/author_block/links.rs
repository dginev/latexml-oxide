//! Link lines in an author block: URLs, link words, mailbox links.

use super::*;

/// A link line ([`link_line_link`]) to a mailbox (`\textbf{Correspondence:} \href{mailto:x@y}{x@y}`, `Email:
/// \href{mailto:…}{…}`), read as a labelled e-mail line (user ruling 2026-10-20, batch 77p r8: 2601.00240, 2601.07974,
/// 2604.23002, 2606.03014, 2605.18563): the text before the link, as it prints without its font ("Correspondence:"),
/// is the label, and the link, kept a link, the address ([`address_text`] reads the one it prints). `None` for any
/// other line.
pub(in crate::base_utilities) fn mailto_line(
  line: &Tokens,
) -> Option<(Option<Tokens>, EmailAddress)> {
  let (at, end) = link_line_link(line)?;
  let v = line.unlist_ref();
  // (the link's target and printed text: to a mailbox, or a bare address without `mailto:`, the ACL template's
  // `\textbf{Correspondence:} \href{binzhu@smu.edu.sg}{binzhu@smu.edu.sg}`, 2604.04847, 2605.29064, 2605.29893,
  // 2606.01600)
  let (target, printed) = link_texts(&v[..end], at);
  if !links_a_mailbox(&target, &printed) {
    return None;
  }
  let mut label: Vec<Token> = Vec::new();
  for t in &without_non_printing_arguments(&v[..at]) {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => label.push(*t),
      _ if (t.get_catcode() == Catcode::SPACE || is_spacing(t))
        && label.last().is_some_and(|last| *last != T_SPACE!()) =>
      {
        label.push(T_SPACE!())
      },
      _ => {},
    }
  }
  while label.last() == Some(&T_SPACE!()) {
    label.pop();
  }
  Some((
    (!label.is_empty()).then(|| Tokens::new(label)),
    EmailAddress {
      address: Tokens::new(v[at..end].to_vec()),
      call:    None,
    },
  ))
}
