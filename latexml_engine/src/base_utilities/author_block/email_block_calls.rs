//! The `\lx@add@email` calls of an author block: owners, correspondence, author notes, non-printing arguments.

use super::*;

/// The email calls for `addresses`: under the author's `label`, `\lx@add@email` named by it; without one, the class's
/// own email command as written (`\email{x}`), or `\lx@add@email`; placed by the label `placement` gives when it gives
/// one (`label=author:N`, the creator that numbers, placed once the authors are made, in the order the lines are written;
/// `label=addresses:block`, the block's).
pub(in crate::base_utilities) fn email_calls(
  label: Option<&Tokens>,
  addresses: Vec<EmailAddress>,
  placement: Option<&str>,
) -> Result<Vec<Token>> {
  let mut kv: Vec<Token> = Vec::new();
  if let Some(label) = label {
    kv.extend(mouth::tokenize_internal("name=").unlist());
    kv.push(T_BEGIN!());
    kv.extend(label.unlist_ref().iter().copied());
    kv.push(T_SPACE!());
    kv.push(T_END!());
  }
  if let Some(placement) = placement {
    if !kv.is_empty() {
      kv.push(T_OTHER!(","));
    }
    kv.extend(mouth::tokenize_internal(TeXString::assembled(placement.to_string())).unlist());
  }
  let keyvals = (!kv.is_empty()).then(|| Tokens::new(kv));
  let mut calls = Vec::new();
  for EmailAddress { address, call } in addresses {
    match (&keyvals, call) {
      // no label of the author's: the class's own command, with the name it gives (amsart's "Email address: ")
      (None, Some(call)) => calls.extend(call.unlist()),
      // under the label, the address as the class's command sets it (JHEP's `E-mail: \email{x}`)
      (Some(_), Some(call)) => {
        let address = match call.unlist_ref().first() {
          Some(cmd) => email_command_address(cmd, address)?,
          None => address,
        };
        calls.extend(
          Invocation!(T_CS!("\\lx@add@email"), vec![
            keyvals.clone(),
            Some(address)
          ])
          .unlist(),
        )
      },
      _ => calls.extend(
        Invocation!(T_CS!("\\lx@add@email"), vec![
          keyvals.clone(),
          Some(address)
        ])
        .unlist(),
      ),
    }
  }
  Ok(calls)
}

/// The authors (indices into `author_marks`) whose marks request the affiliation label `label` ([`mark_label`]).
pub(in crate::base_utilities) fn authors_requesting(
  author_marks: &[Vec<Vec<Token>>],
  label: &str,
) -> Vec<usize> {
  author_marks
    .iter()
    .enumerate()
    .filter(|(_, marks)| {
      marks.iter().any(|operand| {
        clean_frontmatter_labels(&Tokens::new(operand.clone()).to_string(), "affiliation")
          .iter()
          .any(|requested| requested == label)
      })
    })
    .map(|(at, _)| at)
    .collect()
}

/// Who the addresses of a labelled email line may belong to besides the authors they spell ([`block_email_calls`]).
pub(in crate::base_utilities) struct EmailOwners {
  /// the one author a `\thanks` names corresponding (as `author:N` numbers it, from 0)
  pub(in crate::base_utilities) corresponding:   Option<usize>,
  /// the authors (numbered so) the mark of a note just before the line names, a note naming the corresponding author
  pub(in crate::base_utilities) marked:          Vec<usize>,
  /// the authors numbered before this call's (`author:N` counts them)
  pub(in crate::base_utilities) numbered_before: usize,
}

/// The `\lx@add@email` calls for a labelled email line of a marked author block ([`labelled_emails`]): each address to
/// the author whose surname its local part spells (`label=author:N`, [`address_owner`]), as an unmarked block's lines
/// give theirs (counted among all numbered authors, those of earlier calls too); one spelling no name, after a note
/// naming the corresponding author, to the one author that note's mark names; a lone one to the one author a `\thanks`
/// names corresponding (JINST's
/// `S.~Bityukov$^a$\thanks{Corresponding author.}` over `E-mail: \email{Serguei.Bitioukov@cern.ch}`, 1302.2651); else a
/// contact of the whole block (`label=addresses:block`, the shared creator of #159). The label as written is the
/// contact's `name`. (Here the spelling outranks a note's mark; a mark on the address itself outranks the spelling for a
/// contact placed apart from the names, [`placed_contact_owner`].)
pub(in crate::base_utilities) fn block_email_calls(
  label: Option<&Tokens>,
  addresses: Vec<EmailAddress>,
  author_keys: &[NameKeys],
  owners: EmailOwners,
) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  let lone = addresses.len() == 1;
  for EmailAddress { address, call } in addresses {
    // the address itself, not the class's email command around it (`\email{x}` it does not count as one): the contact
    // is what that command stands for, and a class that kills it once its title page is typeset (JINST copies as
    // `arxiv.cls`, `\global\let\email\@gobble`) left the contact empty (1111.1180, 1210.0427); set as the command
    // sets it ([`email_command_address`])
    let (address, command) = match whole_line_cs_wrapper(&address) {
      Some((command, inner)) if is_email_wrapper(&command) => (inner, Some(command)),
      _ => (
        address,
        call.and_then(|call| call.unlist_ref().first().copied()),
      ),
    };
    let mut keyvals = Vec::new();
    if let Some(label) = label {
      keyvals.extend(mouth::tokenize_internal("name=").unlist());
      keyvals.push(T_BEGIN!());
      keyvals.extend(label.unlist_ref().iter().copied());
      keyvals.push(T_SPACE!());
      keyvals.push(T_END!());
      keyvals.push(T_OTHER!(","));
    }
    // the author the address spells; spelling no one, a lone address is the corresponding author's; spelling several
    // (`X.~Wang`, `Y.~Wang`), the corresponding author's only if among them
    let spelled: Vec<usize> = address_owners(author_keys, &address)
      .into_iter()
      .filter(|_| !lists_local_parts(&address_text(&address)))
      .map(|author| owners.numbered_before + author)
      .collect();
    let owner = match spelled.as_slice() {
      [owner] => Some(*owner),
      [] => owners.corresponding.filter(|_| lone),
      several => owners
        .corresponding
        .filter(|author| several.contains(author)),
    };
    // spelled by one name: that author's, though a note marked otherwise precedes it (1710.00802's `aleder@` is A.
    // Leder's, under "Corresponding Author"); else after a note naming the corresponding author, the one author its
    // mark names (1305.7027's `$^\spadesuit$`)
    // (a lone address only, and among several it spells only one of them)
    let owner = match (spelled.len(), owners.marked.as_slice()) {
      (1, _) => owner,
      (0, [marked]) if lone => Some(*marked),
      (_, [marked]) if lone && spelled.contains(marked) => Some(*marked),
      _ => owner,
    };
    let placement = match owner {
      // (`author:N` counts every numbered author, those of an earlier `\author` call too: 1805.09245's collaboration)
      Some(author) => s!("label=author:{}", author + 1),
      None => s!("label=addresses:block"),
    };
    keyvals.extend(mouth::tokenize_internal(TeXString::assembled(placement)).unlist());
    let address = match command {
      Some(command) => email_command_address(&command, address)?,
      None => address,
    };
    calls.extend(
      Invocation!(T_CS!("\\lx@add@email"), vec![
        Some(Tokens::new(keyvals)),
        Some(address)
      ])
      .unlist(),
    );
  }
  Ok(calls)
}

/// A labelled email line ([`labelled_emails`]) behind what prints nothing before it — spacing, declarations, the
/// paragraph break of a blank line in the block (1112.1037's `…Davis,\\ <blank line> E-mail: \email{…}`) — without
/// that lead or a like tail, or `None`.
pub(in crate::base_utilities) fn labelled_email_line(line: &Tokens) -> Result<Option<Tokens>> {
  let toks = line.unlist_ref();
  let mut lead = 0;
  loop {
    let start = lead;
    while toks.get(lead).is_some_and(|t| *t == T_CS!("\\par")) {
      lead += 1;
    }
    lead += leading_unprinted_end(&toks[lead..]);
    if lead == start {
      break;
    }
  }
  // …and a blank line or spacing after it (1311.3535's `E-mail: \email{…}<blank line>}`)
  let mut end = toks.len();
  while end > lead && (toks[end - 1] == T_CS!("\\par") || is_spacing(&toks[end - 1])) {
    end -= 1;
  }
  // …and a group around the whole of it, with what leads it there (`\tt\small {Email Address:
  // songyuxinbb@outlook.com}`, 2603.19228)
  // (`\small {…}` reads as a command over its argument, which a size or font command does not take)
  loop {
    let open = if toks.get(lead).is_some_and(|t| {
      t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s))
    }) {
      lead + 1
    } else {
      lead
    };
    if end > open + 1 && toks[open].get_catcode() == Catcode::BEGIN && skip_group(toks, open) == end
    {
      lead = open + 1 + leading_unprinted_end(&toks[open + 1..end - 1]);
      end -= 1;
    } else {
      break;
    }
  }
  let rest = Tokens::new(toks[lead..end].to_vec());
  Ok(labelled_emails(&rest)?.is_some().then_some(rest))
}

/// The label and addresses of a labelled e-mail line ([`labelled_emails`]) behind what prints nothing before it
/// ([`labelled_email_line`]); `None` for any other line.
pub(in crate::base_utilities) fn line_labelled_emails(
  line: &Tokens,
) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  match labelled_email_line(line)? {
    Some(rest) => labelled_emails(&rest),
    None => Ok(None),
  }
}

/// A class's affiliation line that holds only e-mail addresses — behind its label, `e-mail: kangx@shao.ac.cn`, or a bare
/// `x@y` (`line_labelled_emails`, `line_is_email`) —, made the class's e-mails apart from the names: each address a
/// `\lx@add@contact@placed[role=email]`, the label its `name`, placed as such a contact is (the author the address
/// spells, …, else the block's; `placed_contact_owner`), `mark` the operand of the mark the line led with (`$^*$`, the
/// author requesting it first). `None` for any other line, a line of the class's own e-mail
/// commands too (those make their contacts themselves). AASTeX's `\affil{e-mail: kangx@shao.ac.cn}` after `\author{X.
/// Kang$^{1,2}$, …}` (astro-ph0408475) is X. Kang's e-mail, not an affiliation of the authors.
pub fn placed_email_line_calls(line: &Tokens, mark: &Tokens) -> Result<Option<Tokens>> {
  let (label, addresses) = match line_labelled_emails(line)? {
    Some((label, addresses)) if addresses.iter().all(|address| address.call.is_none()) => (
      label,
      addresses
        .into_iter()
        .map(|address| address.address)
        .collect(),
    ),
    Some(_) => return Ok(None),
    None if line_is_email(line) => (None, vec![line.clone()]),
    None => return Ok(None),
  };
  let mut calls = Vec::new();
  for address in addresses {
    let mut keyvals = mouth::tokenize_internal("role=email").unlist();
    if let Some(label) = &label {
      keyvals.extend(mouth::tokenize_internal(",name=").unlist());
      keyvals.push(T_BEGIN!());
      keyvals.extend(label.unlist_ref().iter().copied());
      keyvals.push(T_SPACE!());
      keyvals.push(T_END!());
    }
    calls.extend(
      Invocation!(T_CS!("\\lx@add@contact@placed"), vec![
        Some(Tokens::new(keyvals)),
        Some(mark.clone()),
        Some(address)
      ])
      .unlist(),
    );
  }
  Ok(Some(Tokens::new(calls)))
}

/// A class's affiliation line without the e-mail commands written in it, and those e-mails as the class's e-mails apart
/// from the names (`placed_email_line_calls`: the author the address spells, …, else the block's): an address set in a
/// line that goes to several authors by its label is one author's, not each holder's (`\affil{$^1$ Key Laboratory …,
/// Nanjing 210008, China\email{zhangqm@pmo.ac.cn}}`, 1503.02933: Q. M. Zhang's, not the five authors' of line 1).
pub fn take_placed_line_emails(line: &Tokens) -> Result<(Tokens, Tokens)> {
  let tokens = line.unlist_ref();
  let (mut kept, mut calls) = (Vec::new(), Vec::new());
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    if t.get_catcode() == Catcode::CS && is_email_command(&t)? {
      let mut j = i + 1;
      while tokens.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      if tokens.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[j..].iter().position(|t| *t == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if let Some((address, next)) = sup_operand_at(tokens, j)
        && let Some(placed) =
          placed_email_line_calls(&Tokens::new(address), &Tokens::new(Vec::new()))?
      {
        calls.extend(placed.unlist());
        i = next;
        continue;
      }
    }
    kept.push(t);
    i += 1;
  }
  Ok((Tokens::new(kept), Tokens::new(calls)))
}

/// The one author (index among `authors`) whose own `\thanks`/`\footnote` note names them the corresponding author
/// ("Corresponding author.", "Correspondence to …"), or `None` when no author's or several authors' do.
pub(in crate::base_utilities) fn corresponding_author<'a>(
  authors: impl Iterator<Item = &'a Tokens>,
) -> Option<usize> {
  match author_notes(authors, &[T_CS!("\\thanks"), T_CS!("\\footnote")])
    .iter()
    .filter(|note| note.names_correspondence)
    .map(|note| note.owner)
    .collect::<std::collections::BTreeSet<usize>>()
    .into_iter()
    .collect::<Vec<_>>()
    .as_slice()
  {
    [owner] => Some(*owner),
    _ => None,
  }
}

/// A note an author's own name holds ([`author_notes`]).
pub(in crate::base_utilities) struct AuthorNote {
  /// the author (index among the names read) whose name holds it
  pub(in crate::base_utilities) owner:                usize,
  /// for a jheppub-family `\note`, which prints its text under a mark it shows beside no name (jheppub.sty:84-89), the
  /// label of that mark, which the authors it is about write on their names: its optional argument's (`\note[*]{…}`),
  /// else the n-th such note's number, the footnote counter it steps (`\author[a,1]{M.V. Smirnov,\note{Corresponding
  /// author.}}` … `\author[a,1]{{\rm and} J.J. Ling}`, 1802.03677); `None` for a `\thanks` or `\footnote`, which
  /// prints its own mark on its owner's name
  pub(in crate::base_utilities) mark:                 Option<String>,
  /// whether `mark` is the counter's number rather than the note's own optional argument: a number an affiliation's
  /// label also is, the authors who write it request that affiliation (`\author[1]{Ann Able,\note{…}}`
  /// `\author[1]{Bob Baker}` `\affiliation[1]{Univ A}`: Bob Baker's 1 is Univ A's), so it is no legend of the note's
  pub(in crate::base_utilities) numbered:             bool,
  /// whether its text names the corresponding author ([`names_correspondence`])
  pub(in crate::base_utilities) names_correspondence: bool,
}

/// The notes of the commands `notes` that the names `authors` hold, in their order (a binding's own note command too,
/// jheppub's `\note{Corresponding author.}`, `\note[*]{…}`).
pub(in crate::base_utilities) fn author_notes<'a>(
  authors: impl Iterator<Item = &'a Tokens>,
  notes: &[Token],
) -> Vec<AuthorNote> {
  let mut found = Vec::new();
  let mut numbered = 0;
  for (owner, author) in authors.enumerate() {
    let toks = author.unlist_ref();
    let mut i = 0;
    while i < toks.len() {
      if !notes.contains(&toks[i]) {
        i += 1;
        continue;
      }
      // (past an optional argument: `\thanks[1]{…}`, `\note[*]{…}`)
      let mut at = i + 1;
      let mut mark = None;
      if toks.get(at) == Some(&T_OTHER!("["))
        && let Some(close) = toks[at..].iter().position(|t| *t == T_OTHER!("]"))
      {
        mark = clean_frontmatter_labels(
          &Tokens::new(toks[at + 1..at + close].to_vec()).to_string(),
          "affiliation",
        )
        .into_iter()
        .next();
        at += close + 1;
      }
      if !toks
        .get(at)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      {
        i += 1;
        continue;
      }
      let end = skip_group(toks, at);
      // (a `\thanks`/`\footnote` prints its own mark on the name, whatever its optional number: its owner alone, and
      // its number no legend of marks the authors write — `$^1$` there is an affiliation's)
      let mut counted = false;
      if toks[i] != T_CS!("\\note") {
        mark = None;
      } else if mark.is_none() {
        numbered += 1;
        counted = true;
        mark = clean_frontmatter_labels(&numbered.to_string(), "affiliation")
          .into_iter()
          .next();
      }
      found.push(AuthorNote {
        owner,
        mark,
        numbered: counted,
        names_correspondence: names_correspondence(&visible_name_text(&toks[at..end])),
      });
      i = end;
    }
  }
  found
}

/// The addresses of a line made only of the class's email commands (a macro whose expansion is `\lx@add@email…`,
/// JHEP's `\email`, amsart's) and the commas, semicolons or spaces between them, each with its command call; `None`
/// for any other line.
pub(in crate::base_utilities) fn email_command_calls(
  line: &Tokens,
) -> Result<Option<Vec<EmailAddress>>> {
  let tokens = line.unlist_ref();
  let mut calls = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    let t = &tokens[i];
    if is_spacing(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";") {
      i += 1;
      continue;
    }
    if t.code != Catcode::CS || !is_email_command(t)? {
      return Ok(None);
    }
    let command = i;
    i += 1;
    while i < tokens.len() && tokens[i] == T_SPACE!() {
      i += 1;
    }
    // the command's optional argument, part of the call (AASTeX's `\email[show]{x}`, aastex701.cls:13341)
    if i < tokens.len() && tokens[i] == T_OTHER!("[") {
      let Some(close) = tokens[i..].iter().position(|t| *t == T_OTHER!("]")) else {
        return Ok(None);
      };
      i += close + 1;
      while i < tokens.len() && tokens[i] == T_SPACE!() {
        i += 1;
      }
    }
    if i >= tokens.len() || tokens[i].code != Catcode::BEGIN {
      return Ok(None);
    }
    let start = i + 1;
    let mut depth = 0usize;
    while i < tokens.len() {
      match tokens[i].code {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            break;
          }
        },
        _ => {},
      }
      i += 1;
    }
    if depth != 0 {
      return Ok(None);
    }
    calls.push(EmailAddress {
      address: Tokens::new(tokens[start..i].to_vec()),
      call:    Some(Tokens::new(tokens[command..=i].to_vec())),
    });
    i += 1;
  }
  Ok(if calls.is_empty() { None } else { Some(calls) })
}

/// A note or mark command an author name carries (`\thanks`, `\footnote`, `\inst`, …: [`is_name_note_command`], the
/// [`NAME_ANNOTATIONS`] but those printed in place, [`PRINTED_IN_PLACE`]; elsarticle's note references), or a macro
/// whose expansion adds to the frontmatter itself (`\lx@add@…`, `\lx@annotate@…`, `\@add@frontmatter`, `\lx@request@…`:
/// aa's `\mail`, achemso's `\email`, JHEP's `\email`).
pub(in crate::base_utilities) fn adds_to_frontmatter(cs: &Token) -> Result<bool> {
  let note = cs.with_str(|name| {
    is_name_note_command(name)
      || matches!(
        name,
        "\\footnotemark"
          | "\\footnotetext"
          | "\\fnref"
          | "\\corref"
          | "\\tnoteref"
          | "\\thankstext"
      )
  });
  if note {
    return Ok(true);
  }
  // a macro whose expansion begins with one of those, or with a macro that does (quantumarticle's `\ead` → `\email`);
  // a macro computed by code (imsart's `\ead`) is taken to add itself
  let frontmatter_call = |t: &Token| {
    t.with_str(|name| {
      [
        "\\lx@add@",
        "\\lx@annotate@",
        "\\@add@frontmatter",
        "\\lx@request@",
      ]
      .iter()
      .any(|prefix| name.starts_with(prefix))
    })
  };
  let Some(def) = lookup_definition(cs)? else {
    return Ok(false);
  };
  Ok(match def.get_expansion() {
    Some(ExpansionBody::Tokens(body)) => match body.unlist_ref().first() {
      Some(first) if frontmatter_call(first) => true,
      Some(first) if first.code == Catcode::CS => lookup_definition(first)?.is_some_and(|inner| {
        matches!(inner.get_expansion(), Some(ExpansionBody::Tokens(body))
          if body.unlist_ref().first().is_some_and(frontmatter_call))
      }),
      _ => false,
    },
    Some(_) => true,
    None => false,
  })
}

/// A class's email command: a macro whose expansion begins with a frontmatter email call — `\lx@add@email`
/// (JHEP.cls.ltxml's `\email`), or one that decides how to record the address (AASTeX 7's `\lx@add@aas@email`, the
/// `\lx@add@email@metadata` of an address the PDF does not print).
pub(in crate::base_utilities) fn is_email_command(cs: &Token) -> Result<bool> {
  let Some(def) = lookup_definition(cs)? else {
    return Ok(false);
  };
  Ok(
    matches!(def.get_expansion(), Some(ExpansionBody::Tokens(body))
    if body.unlist_ref().first().is_some_and(|first| first.with_str(|name| {
      name.strip_prefix("\\lx@add@").is_some_and(|rest| rest.contains("email"))
    }))),
  )
}

/// A marker-less line that is *purely* a list of email addresses — a shared
/// `\texttt{a@x, b@y,}` / `\email{...}` / `\{a,b\}@x` line covering all authors. Like
/// `line_is_email` but tolerates a comma-separated list (and the trailing
/// comma authors often leave), as `email_addresses` reads it: every item a full
/// address or a braced list of local parts before its domain. A prose affiliation
/// line ("Dept. of Foo, University of Pisa, Italy") is rejected, so this never
/// misclassifies an address as an email.
pub fn line_is_email_list(line: &Tokens) -> bool { email_addresses(line).is_some() }

/// A list of local parts written without spaces before one domain, `text` whole (spaces round it aside), whose every
/// part spells exactly one author, each a different one (`owner`, letter case ignored as the spelling reads it), each
/// part at the domain, as the source writes it: `M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk` under the three
/// names, astro-ph0612088 — a capitalised part, which [`write_out_unspaced_lists`] leaves whole, is a local part when it
/// spells its author. `None` for any other text: an address is never invented.
pub(in crate::base_utilities) fn spelled_unspaced_list(
  text: &str,
  owner: &dyn Fn(&str) -> Option<usize>,
) -> Option<Vec<String>> {
  let text = text.trim();
  if text.contains(|c: char| c.is_whitespace() || "{};|/".contains(c)) {
    return None;
  }
  let (list, domain) = text.split_once('@')?;
  let domain_ok = domain.contains('.')
    && !domain.starts_with(['.', '-'])
    && !domain.ends_with(['.', '-'])
    && domain
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
  let parts: Vec<&str> = list.split(',').collect();
  if !domain_ok || parts.len() < 2 {
    return None;
  }
  let mut owners: Vec<usize> = Vec::with_capacity(parts.len());
  for part in &parts {
    if part.chars().nth(1).is_none()
      || part.starts_with('.')
      || !part.chars().all(is_local_char)
      || !part.chars().any(|c| c.is_ascii_alphabetic())
    {
      return None;
    }
    let one = owner(part)?;
    if owners.contains(&one) {
      return None;
    }
    owners.push(one);
  }
  Some(parts.iter().map(|part| s!("{part}@{domain}")).collect())
}
