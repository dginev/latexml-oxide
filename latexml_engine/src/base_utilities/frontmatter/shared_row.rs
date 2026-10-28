//! A contact a class prints under the whole row of its names: every author's, its e-mail lines by their owners.

use super::*;

mod emails;
use emails::row_email_calls;

/// The label every author of a flush requests, which a shared contact ([`share_row_contacts`]) is placed by.
const SHARED_LABEL: &str = "shared";

/// The label a shared contact is placed by, as its authors request it (`row:shared`): an e-mail digested inside such a
/// contact never inherits it — an address is one author's or the block's, never every author's — but keeps its own
/// label, or else is the author block's (`\lx@annotate@frontmatter@now`).
pub(in crate::base_utilities) const SHARED_ROW_LABEL: &str = "row:shared";

/// Rust-only (OXIDIZED_DESIGN_DIVERGENCES #901): the contacts queued with `_share=authors` — llncs's one `\institute`,
/// no `\and` in it (llncs.cls:868-886: `\institutename` prints a sole institute unnumbered under the names, `\inst{n}`
/// numbers each author's when there are several) — are every author's, as the class prints them under the whole author
/// row, unless an author's `\inst{n}` picks them. Perl numbers it `affiliation:1` (llncs.cls.ltxml:46-49), which no
/// author requests without `\inst{1}`, and its prefix-stripped fallback strands it on the first author
/// (KNOWN_PERL_ERRORS #1195); the kernel had kept it on a nameless creator under the names (OXIDIZED_DESIGN #159), no
/// author having an affiliation (20 of the frontmatter scorer's 56 llncs papers: 1002.2793, 1307.8067, 1405.1167,
/// 1509.00818, 1609.01220, 1809.11127, 1910.11599, 2011.00616, 2101.06781, 2106.04289, 2205.06740, 2207.04574,
/// 2207.11464, 2306.03360, 2405.11276, 2502.08785, 2511.12796, 2603.26486, 2604.20229, 2402.19043; and, marked by
/// hand or by a paper's macro that no one answers, 1804.05870, 2007.07423, 2102.12877, 2403.11865, 2607.02596).
/// Several institutes are numbered and paired by no one (2207.06953): not shared.
///
/// Shaped in the queue, before anything is digested: each author requests the shared label (`row:shared`, at the start
/// of its content, while its entry is the one being digested) and the contact sets it as its own
/// (`\lx@set@frontmatter@label`), so the contact is digested once and placed on every author as an `\inst`-picked one
/// is. Its e-mail calls (the class's email command, [`is_email_command`]), and with several authors its lines holding an
/// address outside them (`{\tt \{a,b\}@x}`, `\url`, `\href{mailto:…}`, a `\urldef`'d macro: 1103.2027, 1307.8067), are
/// lines under the row, each decided on its own ([`row_email_calls`]): each address goes to the author it spells,
/// else, a list of one address per author that every spelled address agrees with, to the author at its place, else,
/// with one author, to that author, else, the one address left of several authors, to the one marked as the
/// corresponding author, else, in a shared contact, to the author block (FIDELITY_CHARTER §4, as
/// [`row_address_owners`] reads eptcs's rows) — `\lx@row@email{label}{address}`, after the contact, where a class may
/// set its own e-mail's name. A brace list of local parts (`\{able,baker\}@x.edu`) is each address apart, one address
/// in printed braces that address (`\email{\{fatima.iribu@gmail.com\}}`, 2410.04091); a call the address reader
/// cannot part (`\email{{a,b}@x}`, 2206.10050; a bracket list cut at its commas, 2609.24668) is a shared contact's
/// block's; a label the rule recognises goes with the call (`Corresponding author:`, 2410.06985), any other text stays
/// the institute's. In a picked contact the marks decide its authors: only an address read and given an owner leaves
/// it, everything else stays in it as written, its picked authors' — the block is the fail-safe of the shared path
/// only. Left in a shared contact, an address would have taken the institute's label and gone to every author
/// ([`SHARED_ROW_LABEL`] keeps any e-mail it does not see from that).
///
/// Picks: only the class's own request, a command that requests a labelled annotation (llncs's `\inst{1}`), whose
/// label is the contact's labelseq prefix and a number — then the marks place it, as before. A hand-typed `$^{1}$`
/// (2607.02596), a paper's mark macro (`\affmark{1}`, `\printfnsymbol{1}`: 1804.05870, 2007.07423, 2102.12877,
/// 2403.11865) and llncs's `\fnmsep` are answered by no one: they pick nothing. `\inst{*}` (2207.04574) and an empty
/// `\inst{}` (2604.20229) pick no institute. Not shared either ([`marks_pick`]): an institute holding marks of its own
/// (2608.21425), or under a shown mark numbering another institute than the first (`$^{1,2}$` over `{1} Univ A \\ {2}
/// Univ B`, several numbered by hand).
pub(in crate::base_utilities) fn share_row_contacts(
  commands: &mut Vec<RawFrontmatter>,
) -> Result<()> {
  let is_author = |entry: &RawFrontmatter| {
    entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
  };
  let authors: Vec<usize> = (0..commands.len())
    .filter(|at| is_author(&commands[*at]))
    .collect();
  // (each row contact, and whether it is shared: no `\inst{n}` picks it)
  let mut rows: Vec<(usize, bool)> = Vec::new();
  for at in 0..commands.len() {
    if commands[at].0 == "ltx:contact" && commands[at].1.contains_key("_share") {
      rows.push((at, !marks_pick(commands, &authors, at)?));
    }
  }
  if rows.is_empty() || authors.is_empty() {
    return Ok(());
  }
  // (every author's content group, and every row contact's, or nothing is shaped)
  if authors
    .iter()
    .chain(rows.iter().map(|(at, _)| at))
    .any(|at| content_group(commands[*at].2.unlist_ref()).is_none())
  {
    return Ok(());
  }
  let names: Vec<Tokens> = authors
    .iter()
    .map(|at| queued_name(&commands[*at].2))
    .collect();
  let any_shared = rows.iter().any(|(_, shared)| *shared);
  // Each row contact without its e-mail lines (none when nothing else is left of it), each address's piece with its
  // label, in order.
  let mut shaped: Vec<(usize, Option<Tokens>, PlacedEmails)> = Vec::new();
  for &(at, shared) in &rows {
    let (contact, emails) = row_email_calls(commands, &commands[at].2, &names, shared)?;
    let contact = match contact {
      Some(contact) => {
        let mut contact = contact.unlist();
        if shared {
          let (open, _) = content_group(&contact).unwrap_or((0, 0));
          let set = Invocation!(T_CS!("\\lx@set@frontmatter@label"), vec![
            Some(mouth::tokenize_internal("row")),
            Some(mouth::tokenize_internal(SHARED_LABEL))
          ]);
          contact.splice(open + 1..open + 1, set.unlist());
        }
        Some(Tokens::new(contact))
      },
      None => None,
    };
    shaped.push((at, contact, emails));
  }
  // (nothing shared and no e-mail line taken out: the queue as it was)
  if !any_shared
    && shaped
      .iter()
      .all(|(at, contact, emails)| emails.is_empty() && contact.as_ref() == Some(&commands[*at].2))
  {
    return Ok(());
  }
  // (the e-mail lines after their contact, the last contact's first: the queue's places stay right)
  for (at, contact, emails) in shaped.into_iter().rev() {
    let after = match contact {
      Some(contact) => {
        commands[at].2 = contact;
        at + 1
      },
      None => {
        commands.remove(at);
        at
      },
    };
    for (offset, (label, call, email)) in emails.into_iter().enumerate() {
      let mut attr = TagAttrs::default();
      let role = if email { "email" } else { "affiliation" };
      attr.insert("role".to_string(), role.to_string());
      attr.insert("label".to_string(), label);
      commands.insert(after + offset, ("ltx:contact".to_string(), attr, call));
    }
  }
  // Each author requests its own place's label (`row:at2`), and the shared label when a contact is shared, at the
  // start of its content.
  let authors: Vec<usize> = (0..commands.len())
    .filter(|at| is_author(&commands[*at]))
    .collect();
  for (place, &at) in authors.iter().enumerate() {
    let mut tokens = commands[at].2.clone().unlist();
    let (open, _) = content_group(&tokens).unwrap_or((0, 0));
    let labels = if any_shared {
      s!("{SHARED_LABEL},{}", place_label(place))
    } else {
      place_label(place)
    };
    let request = Invocation!(T_CS!("\\lx@request@frontmatter@annotation"), vec![
      Some(mouth::tokenize_internal("row")),
      Some(mouth::tokenize_internal(TeXString::assembled(labels)))
    ]);
    tokens.splice(open + 1..open + 1, request.unlist());
    commands[at].2 = Tokens::new(tokens);
  }
  Ok(())
}

/// The e-mail lines taken out of a row contact, each piece with the label it is placed by and whether it is an e-mail
/// (else a line no reader parts, an affiliation of the block, [`row_email_calls`]).
type PlacedEmails = Vec<(String, Tokens, bool)>;

/// The label of the author at `place` (from 0) of a shaped flush, as requested: `at1`, `at2`, … (no bare number, which
/// the prefix-stripped fallback of `relocate_annotations` would match against another prefix's number).
fn place_label(place: usize) -> String { s!("at{}", place + 1) }

/// Do marks pick the institutes the row contact at `at` is numbered among: does an author's request for a labelled
/// annotation — the class's own, llncs's `\inst{1}` ([`requests_a_label`]) — read as its labelseq prefix and a number,
/// or does the contact hold a mark of its own (`\affmark{1} City University … \quad \affmark{2} ShanghaiTech University
/// …`, 2608.21425: lines the names' marks pick, not one institute), or does a mark shown on a name number another
/// institute than the first (`Ann Able$^{1,2}$`: the one institute is several, numbered by hand, `{1} Univ A \\ {2}
/// Univ B`; astro-ph0305539's shape, `perfect_kernel_batch63::institute_with_numbered_lines_is_no_marked_list`)? A
/// mark no one answers picks nothing: a hand-typed `$^{1}$` on the names (2607.02596) or a paper's `\affmark{1}` (the
/// 85b review's p18), `\printfnsymbol{1}`'s star (1804.05870), `\fnmsep`'s comma before a note in the institute (r05).
fn marks_pick(commands: &[RawFrontmatter], authors: &[usize], at: usize) -> Result<bool> {
  let content = commands[at].2.unlist_ref();
  for t in content {
    if is_author_mark(t) || raises_a_bare_mark(t)? {
      return Ok(true);
    }
  }
  let prefix = commands[at]
    .1
    .get("labelseq")
    .filter(|prefix| !prefix.is_empty())
    .cloned()
    .unwrap_or_else(|| "affiliation".to_string());
  let numbered = |label: &String| {
    label
      .strip_prefix(prefix.as_str())
      .and_then(|rest| rest.strip_prefix(':'))
      .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
  };
  // (a shown mark's number other than 1: the one institute is several, numbered by hand, `{1} Univ A \\ {2} Univ B`
  // under `Ann Able$^{1,2}$ \and Bob Baker$^{2}$` — no one institute of every author; a 1 alone may be its only one,
  // 2607.02596)
  let several = |label: &String| numbered(label) && !label.ends_with(":1");
  for author in authors {
    let tokens = outside_notes(commands[*author].2.unlist_ref());
    let mut shown = author_mark_operands(&tokens);
    for (i, t) in tokens.iter().enumerate() {
      if requests_a_label(t)?
        && let Some((operand, _)) = sup_operand_at(&tokens, i + 1)
        && clean_frontmatter_labels(&Tokens::new(operand).to_string(), &prefix)
          .iter()
          .any(numbered)
      {
        return Ok(true);
      }
      if raises_a_bare_mark(t)?
        && let Some((operand, _)) = sup_operand_at(&tokens, i + 1)
      {
        shown.push(operand);
      }
    }
    if shown
      .into_iter()
      .flat_map(|operand| clean_frontmatter_labels(&Tokens::new(operand).to_string(), &prefix))
      .any(|label| several(&label))
    {
      return Ok(true);
    }
  }
  Ok(false)
}

/// Is `t` a command requesting a labelled annotation of the names it is written on, the class's own mark
/// (`\def\inst#1{…}` as the binding gives it, `\lx@request@frontmatter@annotation[affiliation]{#1}`)?
fn requests_a_label(t: &Token) -> Result<bool> {
  if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
    return Ok(false);
  }
  Ok(matches!(
    lookup_definition(t)?.as_ref().and_then(|def| def.get_expansion()),
    Some(ExpansionBody::Tokens(body))
      if body.unlist_ref().iter().find(|t| t.get_catcode() != Catcode::SPACE)
        == Some(&T_CS!("\\lx@request@frontmatter@annotation"))
  ))
}

/// Is `t` a command of the paper's own that raises its argument bare, a mark (`\newcommand{\affmark}[1]
/// {\textsuperscript{#1}}`, 2608.21425; `$^{#1}$`, `\ensuremath{^{#1}}`): its body, its braces, math shifts, spaces and
/// `\relax` aside, a superscript or mark command ([`is_author_mark`]) and the parameter. Not llncs's `\fnmsep`
/// (`\unskip$^,$`, a comma before a note), nor a mark of a symbol of the argument (`\printfnsymbol`'s
/// `\textsuperscript{\@fnsymbol{#1}}`).
fn raises_a_bare_mark(t: &Token) -> Result<bool> {
  if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) || is_author_mark(t) {
    return Ok(false);
  }
  let definition = lookup_definition(t)?;
  let Some(ExpansionBody::Tokens(body)) = definition.as_ref().and_then(|def| def.get_expansion())
  else {
    return Ok(false);
  };
  let core: Vec<&Token> = body
    .unlist_ref()
    .iter()
    .filter(|t| {
      !matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::MATH | Catcode::SPACE
      ) && **t != T_CS!("\\relax")
        && **t != T_CS!("\\ensuremath")
    })
    .collect();
  Ok(
    matches!(core.as_slice(), [mark, arg] if is_author_mark(mark) && arg.get_catcode() == Catcode::ARG),
  )
}

/// The span of a queued command's content, its last group (`\lx@add@frontmatter@now{ltx:creator}[role=author]{…}`,
/// `\lx@annotate@frontmatter@now{ltx:creator}{ltx:contact}[…]{…}`): the places of its braces.
fn content_group(tokens: &[Token]) -> Option<(usize, usize)> {
  let close = tokens.len().checked_sub(1)?;
  if tokens[close].get_catcode() != Catcode::END {
    return None;
  }
  let mut depth = 0usize;
  for at in (0..=close).rev() {
    match tokens[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => {
        depth -= 1;
        if depth == 0 {
          return Some((at, close));
        }
      },
      _ => {},
    }
  }
  None
}
