//! E-mail addresses in an author block: labelled e-mails, owners spelled or by elimination.

use super::*;

/// An author-block line holding email addresses behind the author's own label, or given by the class's email command,
/// as its label and addresses:
/// `E-mail: \email{x}` (JHEP.cls/PoS.cls author blocks, astro-ph0611258), `E-mail: \texttt{x@y}`, `\email{x}`. Each
/// address becomes `\lx@add@email`, the label (as written, "E-mail:") its `name`, so it stays markup a stylesheet can
/// show or hide (as the default "Email: " is). Before, the label stood as an affiliation of its own ("E-mail: ") and
/// a lone `\email{x}` line nested its own `\lx@add@email` in another, leaving an empty email contact. A link line to a
/// mailbox is one too, under whatever label it prints (`\textbf{Correspondence:} \href{mailto:…}{…}`, [`mailto_line`]).
/// `None` for any other line.
pub(in crate::base_utilities) fn labelled_emails(
  line: &Tokens,
) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  // (a footnote symbol before the label is the line's mark, no part of it: `$^*$Corresponding author:
  // \href{mailto:…}{…}`, the r5 re-review's u23)
  let line = &Tokens::new(after_symbol_mark(line.unlist_ref()).to_vec());
  let (label, rest) = split_email_label(line);
  if let Some(calls) = email_command_calls(&rest)? {
    return Ok(Some((label, calls)));
  }
  let commands = rest
    .unlist_ref()
    .iter()
    .filter(|t| t.code == Catcode::CS)
    .map(is_email_command)
    .collect::<Result<Vec<bool>>>()?;
  if label.is_some() && line_is_email(&rest) && !commands.contains(&true) {
    return Ok(Some((label, vec![EmailAddress {
      address: rest,
      call:    None,
    }])));
  }
  // (several, a comma or a semicolon between them, each an address or a link to one: `Emails:
  // \href{mailto:a@x}{a@x}, \href{mailto:b@y}{b@y}`, the r5 re-review's u22)
  if label.is_some() && !commands.contains(&true) {
    let items = split_top_level_list(rest.unlist_ref());
    if items.len() > 1
      && items
        .iter()
        .all(|item| line_is_email(item) || mailto_line(item).is_some())
    {
      // (a tie after the comma parts them, no text of the address: `e-mail: a@x, ~~ b@y`)
      return Ok(Some((
        label,
        items
          .into_iter()
          .map(|address| EmailAddress {
            address: Tokens::new(without_row_spacing(address.unlist())),
            call:    None,
          })
          .collect(),
      )));
    }
    // (or a list the address reader parts, a brace list among them: `Email: aable@x.edu; \{bbaker, ccole\}@y.edu`,
    // 2604.10316; the r8 re-review's §B, probes t1, t2 — had been one affiliation)
    if let Some(addresses) = email_addresses(&rest).filter(|addresses| addresses.len() > 1) {
      return Ok(Some((
        label,
        address_tokens(&rest, addresses)?
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
          .collect(),
      )));
    }
  }
  // …and a link line to a mailbox, under any label (`\textbf{Correspondence:} \href{mailto:…}{…}`, [`mailto_line`]),
  // an address each of a brace list it prints ([`mailto_addresses`]); and several such links after one label, a comma or
  // a semicolon between them (`\textbf{Correspondence:} \href{mailto:a@x}{a@x}, \href{mailto:c@x}{c@x}`, 2604.25444;
  // the r8 re-review's §B, probe t6 — had been one affiliation)
  if let Some((label, address)) = mailto_line(line) {
    return Ok(Some((label, mailto_addresses(address))));
  }
  let items = split_top_level_list(line.unlist_ref());
  if items.len() < 2 {
    return Ok(None);
  }
  let mut label = None;
  let mut addresses = Vec::new();
  for (at, item) in items.iter().enumerate() {
    match mailto_line(item) {
      Some((first, address)) if at == 0 => {
        label = first;
        addresses.extend(mailto_addresses(address));
      },
      Some((None, address)) => addresses.extend(mailto_addresses(address)),
      _ => return Ok(None),
    }
  }
  Ok(Some((label, addresses)))
}

/// The addresses a mailbox link prints ([`mailto_line`]): itself, or, when it prints a list of them, each address of it
/// ([`email_addresses`]: `\href{mailto:julia.belikova@skol.tech}{\{julia.belikova, a.panchenko\}@skol.tech}`,
/// 2602.12235; the r7 re-review's finding 7) — the one its target reaches kept the link, the others as they print.
fn mailto_addresses(link: EmailAddress) -> Vec<EmailAddress> {
  let v = link.address.unlist_ref();
  let Some(at) = v.iter().position(|t| *t == T_CS!("\\href")) else {
    return vec![link];
  };
  let (open, close) = link_printed_argument(v, at);
  let Some(addresses) =
    email_addresses(&Tokens::new(v[open..close].to_vec())).filter(|a| a.len() > 1)
  else {
    return vec![link];
  };
  let ((target_open, target_close), _) = link_arguments(v, at);
  let (target, _) = link_texts(v, at);
  let reached = target.trim_start_matches("mailto:").trim().to_lowercase();
  addresses
    .into_iter()
    .map(|address| {
      let printed = mouth::tokenize(TeXString::assembled(address.replace('_', "\\_"))).unlist();
      let tokens = if address.to_lowercase() == reached {
        let mut linked = vec![T_CS!("\\href")];
        linked.extend_from_slice(&v[target_open..target_close]);
        linked.push(T_BEGIN!());
        linked.extend(printed);
        linked.push(T_END!());
        linked
      } else {
        printed
      };
      EmailAddress {
        address: Tokens::new(tokens),
        call:    None,
      }
    })
    .collect()
}

/// `tokens` after the footnote symbol mark that leads them, as [`rewrite_symbol_superscripts`] writes it
/// (`\lx@frontmatter@keepsup{*}\lx@frontmatter@symbolmark{*}`), a size or font before it and the spaces after it;
/// `tokens` when none does.
pub(in crate::base_utilities) fn after_symbol_mark(tokens: &[Token]) -> &[Token] {
  let whole = tokens;
  // (through a group around the whole piece: `{\small $^{\dagger}$\,University of Toronto}`, 2606.28166;
  // `{\it \normalsize $^\sharp$Department of Biostatistics, …}`, 2602.16540 — the r7 re-review's finding 1)
  let tokens = unwrapped(tokens);
  // (and a size or font before it, `\normalsize $^{\dagger}$Department of Statistics`, 2505.23260)
  let lead = tokens
    .iter()
    .take_while(|t| {
      **t == T_SPACE!()
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s)))
    })
    .count();
  if tokens.get(lead) != Some(&T_CS!("\\lx@frontmatter@keepsup")) {
    return whole;
  }
  let mut at = skip_group(tokens, lead + 1);
  if tokens.get(at) == Some(&T_CS!("\\lx@frontmatter@symbolmark")) {
    at = skip_group(tokens, at + 1);
  }
  let at = at.min(tokens.len());
  &tokens[at
    + tokens[at..]
      .iter()
      .take_while(|t| **t == T_SPACE!())
      .count()..]
}

/// `tokens` inside the groups around the whole of them, a one-argument font command's too (`\textit{…}`), and the spaces
/// around those (`{\small $^{\dagger}$\,Univ A \qquad $^{\ddagger}$\,EPFL}`, 2606.28166): what a wrapper group holds,
/// its font aside, is the piece; `tokens` when no group holds them whole.
pub(in crate::base_utilities) fn unwrapped(tokens: &[Token]) -> &[Token] {
  let mut v = tokens;
  loop {
    let start = v.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = v
      .iter()
      .rposition(|t| *t != T_SPACE!())
      .map_or(start, |at| at + 1);
    let inner = &v[start..end.max(start)];
    // (and a one-argument font command around them, as a group: `\textit{$^{\dagger}$Univ A \qquad $^{\ddagger}$Univ
    // B}`, the r8 re-review's finding 8, probe s7)
    let font = usize::from(
      inner.len() >= 3
        && inner[0].get_catcode() == Catcode::CS
        && inner[0].with_str(|s| s.starts_with("\\text") || s == "\\emph")
        && inner[0].with_str(|s| NAME_TEXT_COMMANDS.contains(&s)),
    );
    let group = &inner[font..];
    if group
      .first()
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && skip_group(group, 0) == group.len()
      && group.len() >= 2
    {
      v = &group[1..group.len() - 1];
    } else {
      return v;
    }
  }
}

/// The items of `tokens` at its commas and semicolons outside groups, each without the spaces around it; empty items
/// dropped.
fn split_top_level_list(tokens: &[Token]) -> Vec<Tokens> {
  let mut items: Vec<Tokens> = Vec::new();
  let mut item: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  let mut finish = |item: &mut Vec<Token>| {
    let start = item.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = item
      .iter()
      .rposition(|t| *t != T_SPACE!())
      .map_or(0, |at| at + 1);
    if start < end {
      items.push(Tokens::new(item[start..end].to_vec()));
    }
    item.clear();
  };
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && (*t == T_OTHER!(",") || *t == T_OTHER!(";")) => {
        finish(&mut item);
        continue;
      },
      _ => {},
    }
    item.push(*t);
  }
  finish(&mut item);
  items
}

/// An address's tokens as TeX prints them where a brace is set in math: `$\{$`, `$\lbrace$` the brace `\{`, `$\}$`,
/// `$\rbrace$` the brace `\}` — the math shifts read as characters (a class's verbatim `\email` argument: llncs's
/// `\email{$\{$fargyriou,mikebekos,symvonis$\}$@math.ntua.gr}`, 1106.2694, 2308.00146, 1602.04934) or as shifts —, so
/// a list of local parts in math braces is the list in text braces (`\lx@add@email`, `\lx@split@addresses`; census138
/// ADDR-b, OXIDIZED_DESIGN_DIVERGENCES #972). Anything else in math stays as it is.
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
  let spaces = |from: usize| {
    from
      + v[from.min(v.len())..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count()
  };
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut at = 0;
  while at < v.len() {
    if shift(&v[at]) {
      let inner = spaces(at + 1);
      if let Some(printed) = v.get(inner).and_then(brace) {
        let close = spaces(inner + 1);
        if v.get(close).is_some_and(shift) {
          out.push(printed);
          at = close + 1;
          continue;
        }
      }
    }
    out.push(v[at]);
    at += 1;
  }
  Tokens::new(out)
}

/// One address of an email line, and the class's email command that gave it (`\email{x}`), if any.
pub(in crate::base_utilities) struct EmailAddress {
  pub(in crate::base_utilities) address: Tokens,
  pub(in crate::base_utilities) call:    Option<Tokens>,
}
