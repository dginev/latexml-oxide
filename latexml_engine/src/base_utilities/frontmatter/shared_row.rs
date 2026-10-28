//! A contact a class prints under the whole row of its names: every author's, its e-mail lines by their owners.

use super::*;

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

/// The notes whose argument is no line of the contact: an e-mail in a `\thanks` stays in the note ([`row_email_calls`]).
const NOTE_COMMANDS: [&str; 3] = ["\\thanks", "\\footnote", "\\footnotetext"];

/// The separators that print nothing of their own before an e-mail call, which go with it: spaces, commas, semicolons,
/// a line break, spacing.
fn is_call_separator(t: &Token) -> bool {
  matches!(t.get_catcode(), Catcode::SPACE)
    || *t == T_OTHER!(",")
    || *t == T_OTHER!(";")
    || *t == T_ACTIVE!('~')
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|name| {
        matches!(
          name,
          "\\\\" | "\\newline" | "\\quad" | "\\qquad" | "\\ " | "\\enspace" | "\\hfill"
        )
      }))
}

/// Does `line` hold a note ([`NOTE_COMMANDS`]), at any depth?
fn holds_a_note(line: &[Token]) -> bool {
  line
    .iter()
    .any(|t| t.get_catcode() == Catcode::CS && t.with_str(|name| NOTE_COMMANDS.contains(&name)))
}

/// The end of the spacing at `tokens[from..to]`'s start that prints nothing of its own: separators
/// ([`is_call_separator`]) and `\hspace{…}`, `\hspace*{…}`.
fn past_spacing(tokens: &[Token], mut from: usize, to: usize) -> usize {
  loop {
    if from < to && is_call_separator(&tokens[from]) {
      from += 1;
    } else if from < to && tokens[from] == T_CS!("\\hspace") {
      let mut arg = from + 1;
      if tokens[arg..to].first() == Some(&T_OTHER!("*")) {
        arg += 1;
      }
      if arg >= to || tokens[arg].get_catcode() != Catcode::BEGIN {
        return from;
      }
      from = skip_group(tokens, arg).min(to);
    } else {
      return from;
    }
  }
}

/// One e-mail call of a row contact ([`row_email_calls`]): its span, the separators before it with it; where its
/// command stands; the command; its argument; whether it is written inside a note of the contact (`\thanks{Mail me at
/// \email{…}}`).
struct EmailCall {
  start:   usize,
  at:      usize,
  end:     usize,
  cs:      Token,
  arg:     Vec<Token>,
  in_note: bool,
}

/// The e-mail calls in `tokens[from..to]`: each class e-mail command ([`is_email_command`]) and its argument group, with
/// the separators before it ([`is_call_separator`]); a note's are taken apart (`in_note`) when `notes` is set, else
/// the note is passed over.
fn email_calls_in(
  tokens: &[Token],
  from: usize,
  to: usize,
  notes: bool,
  in_note: bool,
  calls: &mut Vec<EmailCall>,
) -> Result<()> {
  let mut at = from;
  while at < to {
    let t = tokens[at];
    if t.get_catcode() == Catcode::CS && t.with_str(|name| NOTE_COMMANDS.contains(&name)) {
      let mut next = at + 1;
      while tokens.get(next) == Some(&T_SPACE!()) {
        next += 1;
      }
      if tokens.get(next) == Some(&T_OTHER!("[")) {
        next = tokens[next..to]
          .iter()
          .position(|t| *t == T_OTHER!("]"))
          .map_or(to, |end| next + end + 1);
      }
      let end = skip_group(tokens, next).min(to);
      if notes && next < end && tokens[next].get_catcode() == Catcode::BEGIN {
        email_calls_in(tokens, next + 1, end - 1, notes, true, calls)?;
      }
      at = end;
      continue;
    }
    if t.get_catcode() == Catcode::CS && is_email_command(&t)? {
      let mut arg = at + 1;
      while tokens.get(arg) == Some(&T_SPACE!()) {
        arg += 1;
      }
      if arg < to && tokens[arg].get_catcode() == Catcode::BEGIN {
        let end = skip_group(tokens, arg).min(to);
        let mut start = at;
        while !in_note && start > from && is_call_separator(&tokens[start - 1]) {
          start -= 1;
        }
        // (and the label printed before it, which the contact's own name says: `Email: \email{…}`, 2102.12877)
        if !in_note && let Some(label) = email_label_before(tokens, from, start) {
          start = label;
          while start > from && is_call_separator(&tokens[start - 1]) {
            start -= 1;
          }
        }
        calls.push(EmailCall {
          start,
          at,
          end,
          cs: t,
          arg: tokens[arg + 1..end - 1].to_vec(),
          in_note,
        });
        at = end;
        continue;
      }
    }
    at += 1;
  }
  Ok(())
}

/// What a row contact's e-mail lines are taken out as ([`row_email_calls`]): an e-mail call, or a whole line of the
/// contact. Its span in the contact (the separators before it with it); the addresses read from it, if it is read; its
/// tokens as one piece; each address apart, when it may be parted; whether it is an e-mail (else a line no reader
/// parts, which is the block's as written, an affiliation as the institute is).
struct EmailUnit {
  start: usize,
  end:   usize,
  read:  Option<Vec<String>>,
  whole: Tokens,
  parts: Option<Vec<Tokens>>,
  email: bool,
}

/// A piece of a row contact's e-mail lines ([`row_email_calls`]): its unit, its owner, its address when read alone, its
/// tokens, whether an e-mail.
type EmailPiece = (usize, Option<usize>, Option<String>, Tokens, bool);

/// A row contact's command without its e-mail lines, and each address's piece with the label it is placed by
/// (`row:atN`, or `addresses:block`), in order ([`share_row_contacts`]). `None` for the command when nothing of the
/// contact is left.
///
/// Its lines (parted by `\\`, `\newline`, `\par`, [`content_lines`]) with several authors: a line holding an `@`
/// outside the class's e-mail calls — at any depth, a note's too, or in what a command of the paper's own on it
/// prints ([`holds_raw_address`]: `{\tt \{joxan,navas,andrews\}@…}`, 1103.2027; `\url{{farazi, pallgeuer}@…}`,
/// 1809.11127; `\href{mailto:…}{…}`, 2408.03209; `\urldef`'s `\mailsa`, 1307.8067, and `\mailk, \mailt`, 1408.0384) —
/// is an e-mail line of the row, read with the address reader ([`email_addresses`], after an `Email:` label or a
/// label of its own, `Corresponding author:`, [`own_label_end`]), each address its owner's as the class's calls are; a
/// line of a shared contact the reader cannot read is the block's as written — and when it is the contact's first
/// line, the whole contact is (1706.02124). Left in a shared contact, its text would have been every author's (the 85b
/// re-review's finding 1: 0712.2737, 1103.2027, 1307.8067, 1405.0417, 1408.0384, 1503.02007, 1504.01099, 1511.00725,
/// 1705.06516, 1706.02124, 1708.06860, 1805.11291, 1809.11127, 2103.08389, 2207.07506, 2407.18381, 2408.03209,
/// 2411.00887, 2504.06600). Of the other text on a call's
/// line only a label the rule recognises goes with the call ([`call_label`]: `Corresponding author: \email{…}`,
/// 2410.06985, 2609.00838; `\email{…} (Corresponding author),`, 2608.24697), to the call's owner or with it to the
/// block; the rest stays the institute's, every author's — a label of a form the rule does not know too (a residual,
/// OXIDIZED_DESIGN_DIVERGENCES #901).
///
/// Each other call is decided on its own: one whose addresses the reader takes ([`email_addresses`]) is each address
/// its owner's ([`row_address_owners`]; by spelling alone, [`row_address_spelled_owners`], when another call or line
/// is not read, so no list is known to be one per author), each address whole ([`is_whole_address`]), and one address
/// in printed braces is that address (`\email{\{fatima.iribu@gmail.com\}}`, 2410.04091, [`single_braced`]); one it
/// cannot part (`{jsliwinski,wattenhofer}@ethz.ch`, 2206.10050) is the call as written, the block's — or, one author,
/// that author's. The one address left on the block of several authors goes to the one author marked as the
/// corresponding author ([`marked_corresponding_author`]) who owns no other address of the line (2510.12629's
/// `kgwo0528@` stays the block's beside Jinwoo Kim's `jinwookim@`), unless it is a template's
/// (`{firstname.lastname}@x.edu`, [`is_template_address`]). A shared contact's call inside its note (`\thanks{…
/// \email{…}}`) stays where it is written, given its owner's label in place (the note is every author's; the address
/// is its owner's, else the block's).
///
/// A picked contact (`shared` unset: its authors' `\inst{1}` marks pick it) is its picked authors' as the marks decide:
/// the block is the fail-safe of the shared path only. A piece of it leaves the contact only when it is an address the
/// reader reads and an owner is found for ([`row_address_owners`], the corresponding author), with the label the rule
/// recognises; everything else stays where it is written, its picked authors' (int114a's placement): a line the reader
/// does not read, first or later (`Univ X\\ Twitter: @univx`), and the calls on it, each read apart (`\email{…},
/// Twitter: @univx`); a line holding a note; a call or line no address of which an owner is found for (2109.05583's
/// template); a label of a form the rule does not know; the text left between (the 85b r3 and r4 re-reviews'
/// n31-n33, q01, q02, q19, q25, q31). The one exception is decision 3 of 2026-10-22 (a group an entry's e-mail prints
/// is written out, an unspelled address the block's): of a call or line parted by owner, an address no one is found
/// for is the block's, as in a shared contact (2510.12629's `kgwo0528@` beside Jinwoo Kim's `jinwookim@`). Notes are
/// passed over, their calls in place.
fn row_email_calls(
  commands: &[RawFrontmatter],
  command: &Tokens,
  names: &[Tokens],
  shared: bool,
) -> Result<(Option<Tokens>, PlacedEmails)> {
  let tokens = command.unlist_ref();
  let Some((open, close)) = content_group(tokens) else {
    return Ok((Some(command.clone()), Vec::new()));
  };
  let mut calls: Vec<EmailCall> = Vec::new();
  email_calls_in(tokens, open + 1, close, shared, false, &mut calls)?;
  let label = |owner: Option<usize>| match owner {
    Some(place) => s!("row:{}", place_label(place)),
    None => "addresses:block".to_string(),
  };
  let read = |call: &EmailCall| -> Result<Option<Vec<String>>> {
    let read_tokens = |tokens: &[Token]| -> Result<Option<Vec<String>>> {
      Ok(
        email_addresses(&Invocation!(call.cs, vec![Some(Tokens::new(
          tokens.to_vec()
        ))]))
        .filter(|list| list.iter().all(|address| is_whole_address(address))),
      )
    };
    if let Some(list) = read_tokens(&call.arg)? {
      return Ok(Some(list));
    }
    // (one address in printed braces, `\{fatima.iribu@gmail.com\}`, 2410.04091: that address, set as written)
    Ok(match single_braced(&call.arg) {
      Some(inner) => read_tokens(inner)?.filter(|list| list.len() == 1),
      None => None,
    })
  };
  // (an address apart, as the class would set it — a list of local parts keeping its printed braces,
  // `{stomforde|bsick}@uni-kassel.de`, 1701.08125)
  let written = |address: &String| -> Tokens {
    let written = match address.split_once('@') {
      Some((local, domain)) if lists_local_parts(address) => s!("\\{{{local}\\}}@{domain}"),
      _ => address.clone(),
    };
    mouth::tokenize(TeXString::assembled(written.replace('_', "\\_")))
  };
  // (a call with the label of its own before it, `Corresponding author: \email{…}`, or a parenthetical after it,
  // `\email{…} (Corresponding author),`, [`call_label`]: the label goes with the call, in its piece — the call is not
  // parted, so the label is never every author's nor one owner's of another's address)
  let call_unit = |call: &EmailCall,
                   (floor, start, end): (usize, usize, usize),
                   first: bool|
   -> Result<EmailUnit> {
    let list = read(call)?;
    let (before, after) = call_label(tokens, call, (floor, start, end), first);
    let address = email_command_address(&call.cs, Tokens::new(call.arg.clone()))?;
    let labelled = before.is_some() || after.is_some();
    let mut whole: Vec<Token> = Vec::new();
    if let Some((_, from, to)) = before {
      // (a tie a space, as printed: `Corresponding~author:`)
      whole.extend(tokens[from..to].iter().map(
        |t| {
          if *t == T_ACTIVE!('~') { T_SPACE!() } else { *t }
        },
      ));
      whole.push(T_SPACE!());
    }
    whole.extend(address.unlist());
    if let Some((from, to, _)) = after {
      whole.push(T_SPACE!());
      whole.extend_from_slice(&tokens[from..to]);
    }
    let parts = match &list {
      Some(list) if list.len() > 1 && !labelled => Some(
        list
          .iter()
          .map(|address| email_command_address(&call.cs, written(address)))
          .collect::<Result<Vec<Tokens>>>()?,
      ),
      _ => None,
    };
    let mut first_taken = before.map_or(call.start, |(taken, ..)| taken.min(call.start));
    while first_taken > floor && is_call_separator(&tokens[first_taken - 1]) {
      first_taken -= 1;
    }
    Ok(EmailUnit {
      start: first_taken.max(floor),
      end: after.map_or(call.end, |(_, _, end)| end),
      read: list,
      whole: Tokens::new(whole),
      parts,
      email: true,
    })
  };
  // The units, in order: each line of several authors' contact holding an address outside the calls, whole; each other
  // call apart, with the label of its own the rule recognises ([`call_label`]).
  let mut units: Vec<EmailUnit> = Vec::new();
  let (inner_open, inner_close) = wrapped_span(tokens, open + 1, close);
  let lines = content_lines(tokens, inner_open, inner_close);
  let first_text = lines.iter().position(|(_, start, end)| {
    tokens[*start..*end]
      .iter()
      .any(|t| t.get_catcode() != Catcode::SPACE)
  });
  // (a line of a picked contact holding an address the reader does not read: no list of the contact is then known to be
  // one per author)
  let mut unread_line = false;
  for (place, &(sep, start, end)) in lines.iter().enumerate() {
    let on_line: Vec<&EmailCall> = calls
      .iter()
      .filter(|call| !call.in_note && (start..end).contains(&call.at))
      .collect();
    let spans: Vec<(usize, usize)> = calls
      .iter()
      .filter(|call| (start..end).contains(&call.at))
      .map(|call| (call.at, call.end))
      .collect();
    let (from, to) = trimmed(tokens, start, end);
    // (a unit takes the separators before it back to the one before it: `Univ A \\ \email{…}` leaves "Univ A")
    let floor = units.last().map_or(inner_open, |unit: &EmailUnit| unit.end);
    if names.len() > 1 && holds_raw_address(tokens, from, to, &spans)? {
      let mut sep = sep;
      while sep > floor && is_call_separator(&tokens[sep - 1]) {
        sep -= 1;
      }
      let unit = raw_line_unit(tokens, sep, from, to)?;
      // (a picked contact's line the reader does not read, or one holding a note, stays where it is written, its
      // picked authors' — the marks decide the contact's authors; its calls are each read apart: `Univ X\\ Twitter:
      // @univx`, `\email{able@x.edu}, Twitter: @univx` under `\inst{1}` marks, the 85b r3 and r4 re-reviews' n31-n33,
      // q01, q02, q19, q25, q31)
      if !shared && (!unit.email || holds_a_note(&tokens[from..to])) {
        unread_line = true;
      } else if Some(place) == first_text && !unit.email {
        // (a shared contact's own first line holding an address no reader parts, `… Hamburg, Germany \mailsa`,
        // 1706.02124: the line is no line apart from the institute — the contact is the block's whole, as written)
        let (from, to) = trimmed(tokens, inner_open, inner_close);
        units = vec![EmailUnit {
          start: inner_open,
          end:   inner_close,
          read:  None,
          whole: Tokens::new(tokens[from..to].to_vec()),
          parts: None,
          email: false,
        }];
        break;
      } else {
        units.push(unit);
        continue;
      }
    }
    for call in on_line {
      let floor = units.last().map_or(inner_open, |unit: &EmailUnit| unit.end);
      units.push(call_unit(
        call,
        (floor, start, end),
        Some(place) == first_text,
      )?);
    }
  }
  if units.is_empty() && calls.iter().all(|call| !call.in_note) {
    return Ok((Some(command.clone()), Vec::new()));
  }
  // Each read address's owner.
  let flat: Vec<String> = units
    .iter()
    .filter_map(|unit| unit.read.clone())
    .flatten()
    .collect();
  let owners: Vec<Option<usize>> = match names {
    [_] => vec![Some(0); flat.len()],
    _ if !unread_line && units.iter().all(|unit| unit.read.is_some()) => {
      row_address_owners(names, &flat)
    },
    _ => row_address_spelled_owners(names, &flat),
  };
  let mut pieces: Vec<EmailPiece> = Vec::new();
  let mut next = 0;
  for (at, unit) in units.iter().enumerate() {
    let Some(list) = &unit.read else {
      let owner = (names.len() == 1).then_some(0);
      pieces.push((at, owner, None, unit.whole.clone(), unit.email));
      continue;
    };
    let owners = &owners[next..next + list.len()];
    next += list.len();
    // (a unit whose addresses all go one way is the unit as written; else each address apart, or, when it may not be
    // parted, the unit whole, the block's)
    if owners.iter().all(|owner| *owner == owners[0]) {
      let address = (list.len() == 1).then(|| list[0].clone());
      pieces.push((at, owners[0], address, unit.whole.clone(), unit.email));
    } else if let Some(parts) = &unit.parts {
      for ((address, owner), part) in list.iter().zip(owners).zip(parts) {
        pieces.push((at, *owner, Some(address.clone()), part.clone(), unit.email));
      }
    } else {
      pieces.push((at, None, None, unit.whole.clone(), unit.email));
    }
  }
  // (the one address left on the block of several authors: the corresponding author's, when one is marked so — not
  // a template's, `{firstname.lastname}@x.edu`, which is no one's: the 85b re-review's s08, s09)
  let left: Vec<usize> = (0..pieces.len())
    .filter(|at| pieces[*at].1.is_none())
    .collect();
  if let ([at], true) = (left.as_slice(), names.len() > 1)
    && let Some(address) = pieces[*at]
      .2
      .clone()
      .filter(|address| !lists_local_parts(address) && !is_template_address(address))
  {
    // (one who owns none of the line's addresses: one who does is no owner of another, 2510.12629's `kgwo0528@`
    // beside the corresponding author's own `jinwookim@`)
    let author = marked_corresponding_author(commands, &address, None);
    if author.is_some_and(|author| pieces.iter().all(|piece| piece.1 != Some(author))) {
      pieces[*at].1 = author;
    }
  }
  // (a picked contact: the marks decide its authors, and the block is no fail-safe of theirs — a unit no address of
  // which its reader finds an owner for stays in the contact as written, its picked authors', as int114a gives it:
  // 2109.05583's template, 2307.04136's unread `\email`; of a group written out by owner, an address no one is found
  // for is the block's, decision 3 of 2026-10-22, as in a shared one: 2510.12629's `kgwo0528@` beside Jinwoo Kim's own
  // `jinwookim@`)
  if !shared {
    let whole: Vec<bool> = (0..units.len())
      .map(|at| {
        pieces
          .iter()
          .filter(|piece| piece.0 == at)
          .all(|piece| piece.1.is_none())
      })
      .collect();
    pieces.retain(|piece| !whole[piece.0]);
    let mut at = 0;
    units.retain(|_| {
      at += 1;
      !whole[at - 1]
    });
    if units.is_empty() {
      return Ok((Some(command.clone()), Vec::new()));
    }
  }
  let mut placed: PlacedEmails = Vec::new();
  for (_, owner, _, content, email) in pieces {
    let call = if email {
      Invocation!(T_CS!("\\lx@row@email"), vec![
        Some(mouth::tokenize_internal(TeXString::assembled(label(owner)))),
        Some(content)
      ])
    } else {
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        Some(mouth::tokenize_internal(TeXString::assembled(s!(
          "label={{{}}}",
          label(owner)
        )))),
        Some(content)
      ])
    };
    placed.push((label(owner), call, email));
  }
  // The contact without the units (and the separators before them; one at the start takes the separators after it),
  // a note's call in place, given its owner's label.
  let taken = |at: usize| {
    units
      .iter()
      .any(|unit| (unit.start..unit.end).contains(&at))
  };
  let mut edits: Vec<(usize, usize, Option<&EmailCall>)> = units
    .iter()
    .map(|unit| (unit.start, unit.end, None))
    .collect();
  edits.extend(
    calls
      .iter()
      .filter(|call| call.in_note && !taken(call.at))
      .map(|call| (call.start, call.end, Some(call))),
  );
  edits.sort_by_key(|edit| edit.0);
  let mut kept: Vec<Token> = Vec::with_capacity(tokens.len());
  let mut from = 0;
  for (start, end, in_note) in edits {
    // (the separators after a unit at the start may be the next unit's own: `\email{a}, \email{b}`)
    kept.extend_from_slice(&tokens[from.min(start)..start]);
    from = end.max(from);
    if let Some(call) = in_note {
      let owner = match (names, read(call)?) {
        ([_], _) => Some(0),
        (_, Some(list)) => {
          let owners = row_address_spelled_owners(names, &list);
          owners
            .first()
            .copied()
            .flatten()
            .filter(|_| owners.iter().all(|owner| *owner == owners[0]))
        },
        (_, None) => None,
      };
      let address = email_command_address(&call.cs, Tokens::new(call.arg.clone()))?;
      kept.extend(
        Invocation!(T_CS!("\\lx@row@email"), vec![
          Some(mouth::tokenize_internal(TeXString::assembled(label(owner)))),
          Some(address)
        ])
        .unlist(),
      );
      continue;
    }
    if start <= inner_open
      || kept
        .last()
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      while from < inner_close && is_call_separator(&tokens[from]) {
        from += 1;
      }
    }
  }
  kept.extend_from_slice(&tokens[from..]);
  // (nothing left of a shared contact, every line of it taken: no contact; a picked one stays, answering its
  // authors' `\inst{1}` marks, which would otherwise show)
  let (kept_open, kept_close) = content_group(&kept).unwrap_or((0, 0));
  let (kept_open, kept_close) = wrapped_span(&kept, kept_open + 1, kept_close);
  let empty = kept[kept_open..kept_close].iter().all(|t| {
    matches!(
      t.get_catcode(),
      Catcode::SPACE | Catcode::BEGIN | Catcode::END
    ) || is_call_separator(t)
  });
  Ok(((!empty || !shared).then(|| Tokens::new(kept)), placed))
}

/// The span of `tokens[from..to]` its lines are in: within the group of the one command that wraps the whole of it
/// (`\lx@affiliation@withinst{…}`, as `\lx@add@affiliation` gives an institute), spaces aside; else the span itself.
fn wrapped_span(tokens: &[Token], from: usize, to: usize) -> (usize, usize) {
  let (mut first, mut last) = (from, to);
  while first < last && tokens[first].get_catcode() == Catcode::SPACE {
    first += 1;
  }
  while last > first && tokens[last - 1].get_catcode() == Catcode::SPACE {
    last -= 1;
  }
  if first + 1 < last
    && tokens[first].get_catcode() == Catcode::CS
    && tokens[first + 1].get_catcode() == Catcode::BEGIN
    && skip_group(tokens, first + 1) == last
  {
    return wrapped_span(tokens, first + 2, last - 1);
  }
  (from, to)
}

/// Is `t` a line break of a contact's lines ([`content_lines`])?
fn is_line_break(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && t.with_str(|name| matches!(name, "\\\\" | "\\newline" | "\\par"))
}

/// The lines of `tokens[from..to]`, parted by a line break outside any group ([`is_line_break`], with `\\`'s `*` and
/// `[…]`): each line's start with the break before it, its own start, and its end. The lines cover the span.
fn content_lines(tokens: &[Token], from: usize, to: usize) -> Vec<(usize, usize, usize)> {
  let mut lines = Vec::new();
  let (mut sep, mut start, mut depth) = (from, from, 0usize);
  let mut at = from;
  while at < to {
    match tokens[at].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && is_line_break(&tokens[at]) => {
        lines.push((sep, start, at));
        sep = at;
        at += 1;
        if tokens[at..to].first() == Some(&T_OTHER!("*")) {
          at += 1;
        }
        if tokens[at..to].first() == Some(&T_OTHER!("[")) {
          at = tokens[at..to]
            .iter()
            .position(|t| *t == T_OTHER!("]"))
            .map_or(to, |end| at + end + 1);
        }
        start = at;
        continue;
      },
      _ => {},
    }
    at += 1;
  }
  lines.push((sep, start, to));
  lines
}

/// `tokens[start..end]` without the separators at either end ([`is_call_separator`]).
fn trimmed(tokens: &[Token], mut start: usize, mut end: usize) -> (usize, usize) {
  while start < end && is_call_separator(&tokens[start]) {
    start += 1;
  }
  while end > start && is_call_separator(&tokens[end - 1]) {
    end -= 1;
  }
  (start, end)
}

/// A call's label before it (where the spacing before it starts, and its span) and parenthetical after it (its span,
/// and where the punctuation after it ends), [`call_label`].
type CallLabel = (Option<(usize, usize, usize)>, Option<(usize, usize, usize)>);

/// The label of a line's own that goes with an e-mail call on the line `tokens[start..end]` ([`row_email_calls`]), no
/// earlier than `floor` (the unit before it): a short label before it ending in a colon (`Corresponding author:`,
/// 2410.06985, 2609.00838; `Contact:`, 2509.05139), where the spacing before it starts and its span; and a
/// parenthetical of words after it (`(Corresponding author)`, 2608.24697; `(Univ Y)`, the r3 re-review's n23), its
/// span and where it ends with the punctuation after it. The label before it, read as printed (a font's group, a `~`,
/// a spacing command: `\textit{Corresponding author:}`, `Corresponding~author:`, `\hspace{1cm}Contact:`, the 85b r3
/// re-review's n14-n16), is one to four words of letters and hyphens, the first a contact's ([`names_a_contact`]:
/// corresponding, correspondence, contact, author, e-mail, mail, address), from the line's start — not the first
/// line's, the institute's own — or from a `,` or `;`, the spacing before it aside ([`past_spacing`]: the label is
/// set from its first word). Any other text beside a call is the line's, the institute's as before: `… F-76000 Rouen,
/// France. \email{…}` (2609.24668), `\hspace{0.5cm}` (2310.06847), and text that only ends in a colon (`Department of
/// Computer Science:`, `Dept. of CS:`, `University of X, Robotics Lab:`, `Lab of A. B. Smith:`: the r3 re-review's n09,
/// n10, n12, n35). So only the label forms the rule recognises go with the call; a label of another form (prose
/// without a colon, `Corresponding author email \email{…}`; a label word the rule does not know, `Reach me at:
/// \email{…}`; one after a `.`, or after other text on its line) stays in the shared institute, every author's — a
/// residual.
fn call_label(
  tokens: &[Token],
  call: &EmailCall,
  (floor, start, end): (usize, usize, usize),
  first: bool,
) -> CallLabel {
  let floor = floor.max(start);
  let is_word = |t: &Token| t.get_catcode() == Catcode::LETTER || *t == T_OTHER!("-");
  // (words of a label, as printed: letters, hyphens and spaces, the first naming a contact)
  let is_label = |printed: &str| {
    let words: Vec<&str> = printed.split_whitespace().collect();
    (1..=4).contains(&words.len())
      && names_a_contact(words[0])
      && printed
        .chars()
        .all(|c| c.is_alphabetic() || c == '-' || c.is_whitespace())
  };
  let mut before = None;
  // (the piece before the call from the line's start — not the first line's, the institute's own — or from the last
  // `,` or `;` outside any group, the other calls of the line aside, read as printed: through a font's group, a `~`, a
  // spacing command, `\textit{Corresponding author:}`, `Corresponding~author:`, `\hspace{1cm}Contact:`)
  let mut colon = call.at;
  while colon > floor && tokens[colon - 1] == T_SPACE!() {
    colon -= 1;
  }
  let (mut from, mut depth) = (None, 0usize);
  for at in (floor..colon).rev() {
    match tokens[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => depth = depth.saturating_sub(1),
      _ if depth == 0 && [T_OTHER!(","), T_OTHER!(";")].contains(&tokens[at]) => {
        from = Some(at + 1);
        break;
      },
      _ => {},
    }
  }
  let from = from.or((floor == start && !first).then_some(start));
  if let Some(taken) = from {
    // (the label without the spacing before it, which prints nothing of its own: `\hspace{1cm}Contact:`)
    let from = past_spacing(tokens, taken, colon);
    let printed = visible_name_text(&tokens[from..colon]);
    // (an `Email:` label alone is the contact's own name, which the call's start already takes, [`email_label_before`])
    if let Some(words) = printed.trim_end().strip_suffix(':')
      && is_label(words)
      && call.start > from
    {
      before = Some((taken, from, colon));
    }
  }
  let mut open = call.end;
  while open < end && tokens[open] == T_SPACE!() {
    open += 1;
  }
  let mut after = None;
  if open < end && tokens[open] == T_OTHER!("(") {
    let close = (open + 1..end).find(|at| tokens[*at] == T_OTHER!(")"));
    if let Some(close) = close
      && tokens[open + 1..close]
        .iter()
        .all(|t| is_word(t) || *t == T_SPACE!())
      && tokens[open + 1..close].iter().any(is_word)
    {
      let mut past = close + 1;
      while past < end && [T_OTHER!(","), T_OTHER!(";"), T_OTHER!(".")].contains(&tokens[past]) {
        past += 1;
      }
      after = Some((open, close + 1, past));
    }
  }
  (before, after)
}

/// What a `\urldef`'d command of the paper's prints, as text (`\urldef{\mailsa}\path|…|`, 1307.8067), if `t` is one.
fn urldef_text(t: &Token) -> Option<String> {
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  let text = t.with_str(|name| lookup_string(&s!("lx@urldef@text@{name}")));
  (!text.is_empty()).then_some(text)
}

/// The body of a command of the paper's own that takes no argument (`\newcommand{\mailsa}{a@x}`), if `t` is one.
fn parameterless_body(t: &Token) -> Result<Option<Tokens>> {
  if !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE) {
    return Ok(None);
  }
  let Some(definition) = lookup_definition(t)? else {
    return Ok(None);
  };
  if definition
    .get_parameters()
    .is_some_and(|params| !params.get_parameters().is_empty())
  {
    return Ok(None);
  }
  Ok(match definition.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
    _ => None,
  })
}

/// Does `tokens[from..to]`, the spans `calls` (the class's e-mail calls) aside, hold an `@` — a character, at any
/// depth, or one a command of the paper's own on it prints, one level into a command without arguments
/// ([`parameterless_body`], an e-mail call in it aside) or a `\urldef`'d one ([`urldef_text`])?
fn holds_raw_address(
  tokens: &[Token],
  from: usize,
  to: usize,
  calls: &[(usize, usize)],
) -> Result<bool> {
  let mut at = from;
  while at < to {
    if let Some((_, end)) = calls.iter().find(|(start, _)| *start == at) {
      at = *end;
      continue;
    }
    let t = &tokens[at];
    if t.get_catcode() != Catcode::CS && t.with_str(|s| s.contains('@')) {
      return Ok(true);
    }
    if urldef_text(t).is_some_and(|text| text.contains('@')) {
      return Ok(true);
    }
    if !is_email_command(t)?
      && let Some(body) = parameterless_body(t)?
    {
      let body = body.unlist_ref();
      let mut inner = Vec::new();
      email_calls_in(body, 0, body.len(), true, false, &mut inner)?;
      let spans: Vec<(usize, usize)> = inner.iter().map(|call| (call.at, call.end)).collect();
      let mut k = 0;
      while k < body.len() {
        if let Some((_, end)) = spans.iter().find(|(start, _)| *start == k) {
          k = *end;
          continue;
        }
        if body[k].get_catcode() != Catcode::CS && body[k].with_str(|s| s.contains('@')) {
          return Ok(true);
        }
        k += 1;
      }
    }
    at += 1;
  }
  Ok(false)
}

/// A line holding an address outside the class's e-mail calls ([`holds_raw_address`]), `tokens[from..to]`, the break
/// before it at `sep`: read with the address reader ([`email_addresses`]) after an `Email:` label, which the contact's
/// own name says ([`split_email_label`]), or a label of its own (`Corresponding author:`, [`own_label_end`]), what a
/// `\urldef`'d command prints read as its text. A line the reader reads is an e-mail line, parted by owner only with no
/// label of its own; a line it cannot read is the block's, as written.
fn raw_line_unit(tokens: &[Token], sep: usize, from: usize, to: usize) -> Result<EmailUnit> {
  let line = Tokens::new(tokens[from..to].to_vec());
  let (email_label, after_label) = split_email_label(&line);
  let (rest, own_label) = match email_label {
    Some(_) => (after_label, false),
    None => match own_label_end(line.unlist_ref()) {
      Some(end) => (Tokens::new(tokens[from + end..to].to_vec()), true),
      None => (line.clone(), false),
    },
  };
  // (the reader's view: what a `\urldef`'d command prints, as its characters)
  let mut substituted = false;
  let mut view: Vec<Token> = Vec::with_capacity(rest.len());
  for t in rest.unlist_ref() {
    match urldef_text(t) {
      Some(text) => {
        substituted = true;
        view.extend(text.chars().map(|c| {
          if c.is_whitespace() {
            T_SPACE!()
          } else {
            T_OTHER!(c.to_string())
          }
        }));
      },
      None => view.push(*t),
    }
  }
  let read = email_addresses(&Tokens::new(view))
    .filter(|list| list.iter().all(|address| is_whole_address(address)));
  let parts = match &read {
    Some(list) if list.len() > 1 && !own_label && substituted => {
      let mut parts = Vec::with_capacity(list.len());
      for address in list {
        let written = match address.split_once('@') {
          Some((local, domain)) if lists_local_parts(address) => s!("\\{{{local}\\}}@{domain}"),
          _ => address.clone(),
        };
        parts.push(Invocation!(T_CS!("\\lx@address@urlfont"), vec![Some(
          mouth::tokenize(TeXString::assembled(written.replace('_', "\\_")))
        )]));
      }
      Some(parts)
    },
    Some(list) if list.len() > 1 && !own_label => Some(address_tokens(&rest, list.clone())?),
    _ => None,
  };
  let email = read.is_some();
  let whole = if email && !own_label { rest } else { line };
  Ok(EmailUnit {
    start: sep,
    end: to,
    read,
    whole,
    parts,
    email,
  })
}

/// The end of a label of a line's own before its addresses, its colon (`Corresponding author:`): words, no `@`, no
/// command, before the line's first colon outside any group, the first a contact's ([`names_a_contact`]; not
/// `Department of CS: {\tt able@x.edu}`, a line the address reader then cannot read, the block's).
fn own_label_end(line: &[Token]) -> Option<usize> {
  let mut text = String::new();
  for (at, t) in line.iter().enumerate() {
    match t.get_catcode() {
      Catcode::LETTER => t.with_str(|s| text.push_str(s)),
      Catcode::SPACE => text.push(' '),
      Catcode::OTHER if *t == T_OTHER!(":") => {
        return text
          .split_whitespace()
          .next()
          .is_some_and(names_a_contact)
          .then_some(at + 1);
      },
      Catcode::OTHER if t.with_str(|s| !s.contains('@')) => t.with_str(|s| text.push_str(s)),
      _ => return None,
    }
  }
  None
}

/// Is `address` a template's, the address form a paper gives no author (`{firstname.lastname}@x.edu`,
/// `first.last@x.edu`, `name.surname@x.edu`; the 85b re-review's s08, s09; 2102.12877, 2109.05583)?
fn is_template_address(address: &str) -> bool {
  let local = address
    .split_once('@')
    .map_or(address, |(local, _)| local)
    .to_lowercase();
  let words: Vec<&str> = local
    .split(|c: char| !c.is_alphanumeric())
    .filter(|word| !word.is_empty())
    .collect();
  words.iter().any(|word| {
    [
      "firstname",
      "lastname",
      "forename",
      "surname",
      "givenname",
      "familyname",
    ]
    .contains(word)
  }) || words == ["first", "last"]
}

/// Whether `address`, read from an e-mail call ([`email_addresses`]), is an address whole: a local part and a domain of
/// labels parted by dots, nothing of the line around it — not a piece of a list the reader cut at its commas
/// (`[simon.barbarit--gaboriau@insa-, …, samia.ainouz@insa-]rouen.fr`, 2609.24668), which the call keeps as written.
fn is_whole_address(address: &str) -> bool {
  let Some((local, domain)) = address.rsplit_once('@') else {
    return false;
  };
  !local.is_empty()
    && !local.contains(['[', ']', '(', ')', '<', '>', '@'])
    && domain.contains('.')
    && domain.split('.').all(|label| {
      !label.is_empty()
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label.chars().all(|c| c.is_alphanumeric() || c == '-')
    })
}

/// The tokens inside the printed braces of `arg` when they are the whole of it (`\{fatima.iribu@gmail.com\}`, spaces
/// aside), and no other printed brace stands within.
fn single_braced(arg: &[Token]) -> Option<&[Token]> {
  let is = |t: &Token, names: [&str; 2]| {
    t.get_catcode() == Catcode::CS && t.with_str(|name| names.contains(&name))
  };
  let open = ["\\{", "\\lbrace"];
  let close = ["\\}", "\\rbrace"];
  let first = arg.iter().position(|t| t.get_catcode() != Catcode::SPACE)?;
  let last = arg
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)?;
  if first >= last || !is(&arg[first], open) || !is(&arg[last], close) {
    return None;
  }
  let inner = &arg[first + 1..last];
  (!inner.iter().any(|t| is(t, open) || is(t, close))).then_some(inner)
}

/// The start of an e-mail label ending at `tokens[..end]` (`Email:`, `E-mail:`, `Emails:`, the colon after the word),
/// no earlier than `from`, printed before an e-mail call it names, which the call's own contact name says.
fn email_label_before(tokens: &[Token], from: usize, end: usize) -> Option<usize> {
  if end <= from || tokens[end - 1] != T_OTHER!(":") {
    return None;
  }
  let mut start = end - 1;
  while start > from
    && (tokens[start - 1].get_catcode() == Catcode::LETTER || tokens[start - 1] == T_OTHER!("-"))
  {
    start -= 1;
  }
  let word: String = tokens[start..end - 1]
    .iter()
    .map(Token::to_string)
    .collect();
  (start < end - 1 && is_email_label(&word)).then_some(start)
}
