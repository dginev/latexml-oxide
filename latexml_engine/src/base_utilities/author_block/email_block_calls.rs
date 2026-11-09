//! The `\lx@add@email` calls of an author block: owners, correspondence, author notes, non-printing arguments.

use super::*;

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

/// A marker-less line that is *purely* a list of email addresses — a shared
/// `\texttt{a@x, b@y,}` / `\email{...}` / `\{a,b\}@x` line covering all authors. Like
/// `line_is_email` but tolerates a comma-separated list (and the trailing
/// comma authors often leave), as `email_addresses` reads it: every item a full
/// address or a braced list of local parts before its domain. A prose affiliation
/// line ("Dept. of Foo, University of Pisa, Italy") is rejected, so this never
/// misclassifies an address as an email.
pub fn line_is_email_list(line: &Tokens) -> bool { email_addresses(line).is_some() }
