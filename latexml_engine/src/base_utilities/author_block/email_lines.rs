//! Author-block e-mail lines and the `\@author` replay definitions; mark operands, row starts and empty marks.

use super::*;

/// The `\lx@add@email` calls for a line of addresses in an author block (`a@x, b@y`, `{a,b}@dom`), placed as
/// `placement` says: one call per address, a whole-line wrapper repeated on each, or the line itself when it holds one
/// address (`\href{mailto:a@x}{a@x}` stays a link); else the line as one contact. OXIDIZED_DESIGN #52(j).
pub(in crate::base_utilities) fn email_line_calls(
  line: Tokens,
  placement: AddressPlacement,
) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  // A wrapper is kept on each address (`\texttt`, `\small`, `\url`), but an email command's (`\email`, `\mailto`,
  // `\emailaddr`) is the contact `\lx@add@email` stands for — nested, it made a second (empty) contact and advanced
  // the `labelseq` count twice, so the second address found no author (llncs `\institute{… \\ \email{a@x, b@y}}`).
  // (the address set as the command sets it, [`email_command_address`])
  let unwrapped = |line: Tokens| match whole_line_cs_wrapper(&line) {
    Some((cmd, inner)) if is_email_wrapper(&cmd) => email_command_address(&cmd, inner),
    _ => Ok(line),
  };
  let addrs = email_addresses(&line).unwrap_or_default();
  if let AddressPlacement::Marked(carriers) = &placement
    && !addrs.is_empty()
  {
    // (one address: the line itself, as written)
    let addresses = if addrs.len() == 1 {
      vec![unwrapped(line.clone())?]
    } else {
      address_tokens(&line, addrs)?
    };
    for address in addresses {
      for author in carriers {
        calls.extend(
          Invocation!(T_CS!("\\lx@add@email"), vec![
            Some(mouth::tokenize_internal(TeXString::assembled(s!(
              "label=author:{author}"
            )))),
            Some(address.clone())
          ])
          .unlist(),
        );
      }
    }
    return Ok(calls);
  }
  let placement = match placement {
    AddressPlacement::Settled(owners) if !addrs.is_empty() && owners.len() == addrs.len() => {
      if owners.iter().all(Option::is_none) {
        AddressPlacement::Shared
      } else if owners.iter().all(Option::is_some) {
        AddressPlacement::Authors(owners.into_iter().flatten().collect())
      } else {
        // (the settled ones to their authors, each other the block's, in the line's order)
        for (address, owner) in address_tokens(&line, addrs)?.into_iter().zip(owners) {
          let placement = match owner {
            Some(n) => s!("label=author:{n}"),
            None => s!("label=addresses:block"),
          };
          calls.extend(
            Invocation!(T_CS!("\\lx@add@email"), vec![
              Some(mouth::tokenize_internal(TeXString::assembled(placement))),
              Some(address)
            ])
            .unlist(),
          );
        }
        return Ok(calls);
      }
    },
    AddressPlacement::Settled(_) | AddressPlacement::Marked(..) => AddressPlacement::Shared,
    placement => placement,
  };
  let options: Vec<Option<Tokens>> = match placement {
    AddressPlacement::Sequence if !addrs.is_empty() => addrs
      .iter()
      .map(|_| Some(mouth::tokenize_internal("labelseq=author")))
      .collect(),
    AddressPlacement::Shared if addrs.len() > 1 => addrs
      .iter()
      .map(|_| Some(mouth::tokenize_internal("label=addresses:block")))
      .collect(),
    AddressPlacement::Authors(authors) if !addrs.is_empty() && authors.len() == addrs.len() => {
      authors
        .into_iter()
        .map(|n| {
          Some(mouth::tokenize_internal(TeXString::assembled(s!(
            "label=author:{n}"
          ))))
        })
        .collect()
    },
    placement => {
      // One contact; an email command's wrapper is the contact itself.
      let opts = matches!(placement, AddressPlacement::Shared)
        .then(|| mouth::tokenize_internal("label=addresses:block"));
      calls
        .extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(unwrapped(line)?)]).unlist());
      return Ok(calls);
    },
  };
  if let [opts] = options.as_slice() {
    calls.extend(
      Invocation!(T_CS!("\\lx@add@email"), vec![
        opts.clone(),
        Some(unwrapped(line)?)
      ])
      .unlist(),
    );
    return Ok(calls);
  }
  for (address, opts) in address_tokens(&line, addrs)?.into_iter().zip(options) {
    calls.extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(address)]).unlist());
  }
  Ok(calls)
}

/// Whether `cmd` is an email command (`\email`, `\mailto`, `\emailaddr`): the contact `\lx@add@email` stands for.
pub(in crate::base_utilities) fn is_email_wrapper(cmd: &Token) -> bool {
  cmd.with_str(|name| name.to_lowercase().contains("mail"))
}

/// The address of a call of the email command `cmd` (`\email{x}`) as that command sets it: in the font the class's
/// command gives its address, `\lx@address@urlfont` (llncs's `\email`, `{\tt#1}` at llncs.cls:897; amsart's, JHEP's)
/// or `\lx@address@textfont` (svjour3's, REVTeX 4.1's), else as written — so a contact the author block makes of the
/// call (`\institute{… \\ \email{a@x}}`, `E-mail: \email{x}`) is set as the command's own is. Only the command's own
/// expansion body is read, one level: a command reaching the address macro through another macro is not seen.
pub(in crate::base_utilities) fn email_command_address(
  cmd: &Token,
  address: Tokens,
) -> Result<Tokens> {
  let fonts = [
    T_CS!("\\lx@address@urlfont"),
    T_CS!("\\lx@address@textfont"),
  ];
  let font = match lookup_definition(cmd)?
    .as_ref()
    .and_then(|def| def.get_expansion())
  {
    Some(ExpansionBody::Tokens(body)) => body
      .unlist_ref()
      .iter()
      .find(|t| fonts.contains(t))
      .copied(),
    _ => None,
  };
  Ok(match font {
    Some(font) => Invocation!(font, vec![Some(address)]),
    None => address,
  })
}

/// The addresses `addrs` of an email `line` as tokens, the line's whole-line wrapper (`\texttt`, `\small`, `\url`)
/// repeated on each — but not an email command's, the contact itself.
pub(in crate::base_utilities) fn address_tokens(
  line: &Tokens,
  addrs: Vec<String>,
) -> Result<Vec<Tokens>> {
  let line_wrapper = whole_line_cs_wrapper(line).map(|(cmd, _)| cmd);
  let wrapper = line_wrapper.filter(|cmd| !is_email_wrapper(cmd));
  // (an email command's address set as the command sets it, [`email_command_address`])
  let email_command = line_wrapper.filter(is_email_wrapper);
  // (the line's links to a mailbox, by the address they reach: each address keeps its link, `\texttt{\{}\href{mailto:
  // Xinhao.Zhang@…}{…xinhao.zhang,} \href{mailto:maxime.peyrard@…}{…}\texttt{\}@…}`, 2604.19440)
  let v = line.unlist_ref();
  let links: Vec<(String, Vec<Token>)> = (0..v.len())
    .filter(|at| v[*at] == T_CS!("\\href"))
    .filter_map(|at| {
      let open = at + 1 + v[at + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      let target = v[open..skip_group(v, open).clamp(open, v.len())].to_vec();
      let (text, _) = link_texts(&v[at..], 0);
      let address = text.strip_prefix("mailto:")?.trim().to_lowercase();
      Some((address, target))
    })
    .collect();
  addrs
    .into_iter()
    .map(|addr| {
      let mut toks = mouth::tokenize(TeXString::assembled(addr.replace('_', "\\_"))).unlist();
      if let Some(cmd) = wrapper {
        let mut wrapped = vec![cmd, T_BEGIN!()];
        wrapped.append(&mut toks);
        wrapped.push(T_END!());
        toks = wrapped;
      }
      if email_command.is_none()
        && let Some((_, target)) = links
          .iter()
          .find(|(address, _)| *address == addr.to_lowercase())
      {
        let mut linked = vec![T_CS!("\\href")];
        linked.extend(target.iter().copied());
        linked.push(T_BEGIN!());
        linked.append(&mut toks);
        linked.push(T_END!());
        toks = linked;
      }
      match email_command {
        Some(cmd) => email_command_address(&cmd, Tokens::new(toks)),
        None => Ok(Tokens::new(toks)),
      }
    })
    .collect()
}

/// The end of `tokens` before the line break and spacing they end with, which print nothing: spaces, `\\`, `\hfill`,
/// `\smallskip`…, `\protect`, `\vspace{…}`, `\hspace*{…}` (2409.06765's `\email vye@berkeley.edu\\ \vspace{-0.6cm}`,
/// read as one line; 2609.08102's `\author[Mech]{\protect\\Johann Guilleminot}`); 0 for a line of only those.
pub(in crate::base_utilities) fn spacing_end(tokens: &[Token]) -> usize {
  let spacing = |t: &Token, names: &[&str]| {
    t.get_catcode() == Catcode::CS && t.with_str(|s| names.contains(&s))
  };
  let mut end = tokens.len();
  loop {
    while end > 0 && tokens[end - 1] == T_SPACE!() {
      end -= 1;
    }
    if end > 0
      && spacing(&tokens[end - 1], &[
        "\\\\",
        "\\hfill",
        "\\smallskip",
        "\\medskip",
        "\\bigskip",
        "\\newline",
        "\\protect",
      ])
    {
      end -= 1;
      continue;
    }
    if end > 0 && tokens[end - 1].get_catcode() == Catcode::END {
      let mut depth = 0usize;
      let mut open = None;
      for j in (0..end).rev() {
        match tokens[j].get_catcode() {
          Catcode::END => depth += 1,
          Catcode::BEGIN => {
            depth -= 1;
            if depth == 0 {
              open = Some(j);
              break;
            }
          },
          _ => {},
        }
      }
      if let Some(open) = open {
        let mut cs = open;
        if cs > 0 && tokens[cs - 1] == T_OTHER!("*") {
          cs -= 1;
        }
        if cs > 0 && spacing(&tokens[cs - 1], &["\\vspace", "\\hspace"]) {
          end = cs - 1;
          continue;
        }
      }
    }
    return end;
  }
}

/// A name line and the address it ends with, as the JMLR family writes an author (`\name Ann Able \email ann@uni.edu`,
/// jmlr2e.sty:271-273; 2405.13980, 2407.04153, 2411.04991, 2502.14381, 2511.15722): the name, and the line's last run
/// without a space, the class's email markup with it (`\email ann@uni.edu`), when that run prints one plain address and a
/// name stands before it, its markup leading the address. A run with a note or a command that adds to the frontmatter
/// itself (`\thanks{ann@uni.edu}`, `\footnote{…}`, a class's `\email{x}` expanding to `\lx@add@email`, aa's `\mail`
/// adding a contact) is the author's as it stands, and a line that is only an address is kept whole.
pub(in crate::base_utilities) fn name_and_trailing_email(
  line: &Tokens,
) -> Result<(Tokens, Option<Tokens>)> {
  let tokens = line.unlist_ref();
  let end = spacing_end(tokens);
  // the last run without a space outside braces
  let mut start = end;
  let mut depth = 0usize;
  while start > 0 {
    match tokens[start - 1].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN if depth == 0 => break,
      Catcode::BEGIN => depth -= 1,
      Catcode::SPACE if depth == 0 => break,
      _ => {},
    }
    start -= 1;
  }
  let whole = || Ok((line.clone(), None));
  if start == end || depth != 0 {
    return whole();
  }
  // the markup leads the address (`\email ann@uni.edu`, `\texttt{ann@uni.edu}`): a command or group after printed text
  // is not an address's (`Peng\thanks{bp2601@columbia.edu}`, 2402.08164, is a name and its note)
  let mut printed = false;
  for t in &tokens[start..end] {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => printed = true,
      Catcode::CS
        if printed
          && !matches!(
            t.with_str(str::to_string).as_str(),
            "\\_" | "\\textunderscore"
          ) =>
      {
        return whole();
      },
      Catcode::BEGIN | Catcode::ACTIVE if printed => return whole(),
      _ => {},
    }
  }
  let run = Tokens::new(tokens[start..end].to_vec());
  // (a name annotation other than the address's markup glued to it, `\orcidlink{…}ann@x.org`, `\cite{x}ann@x.org`, is
  // the name's, printed beside it: the run is no address, 77n r5 review probes n02, n04)
  if run.unlist_ref().iter().any(|t| {
    t.code == Catcode::CS
      && t.with_str(|name| name != "\\email" && NAME_ANNOTATIONS.contains(&name))
  }) {
    return whole();
  }
  let commands = run
    .unlist_ref()
    .iter()
    .filter(|t| t.code == Catcode::CS)
    .map(adds_to_frontmatter)
    .collect::<Result<Vec<bool>>>()?;
  let plain = |address: &str| {
    address.split_once('@').is_some_and(|(local, domain)| {
      !local.is_empty()
        && local
          .chars()
          .all(|c| c.is_alphanumeric() || "._%+-".contains(c))
        && domain.contains('.')
        && domain
          .chars()
          .all(|c| c.is_alphanumeric() || ".-".contains(c))
    })
  };
  let one_address =
    email_addresses(&run).is_some_and(|addresses| addresses.len() == 1 && plain(&addresses[0]));
  let name = Tokens::new(tokens[..start].to_vec());
  if commands.contains(&true)
    || !one_address
    || visible_name_text(name.unlist_ref()).trim().is_empty()
  {
    return whole();
  }
  let mut name = name.unlist();
  while name.last() == Some(&T_SPACE!()) {
    name.pop();
  }
  Ok((Tokens::new(name), Some(run)))
}

/// The addresses of an author-block line that is only email addresses (`\texttt{a@x, b@y}`, `{a,b}@dom`), each as
/// tokens ([`address_tokens`]); a line of one address is itself (`\href{mailto:a@x}{a@x}` stays a link). `None` for
/// any other line.
pub(in crate::base_utilities) fn bare_email_addresses(
  line: &Tokens,
) -> Result<Option<Vec<Tokens>>> {
  let Some(addrs) = email_addresses(line).filter(|addrs| !addrs.is_empty()) else {
    return Ok(None);
  };
  if addrs.len() == 1 {
    return Ok(line_is_email(line).then(|| vec![line.clone()]));
  }
  address_tokens(line, addrs).map(Some)
}

/// The definitions in force where a class's title code reads `\@author`: the statements of the groups still open there
/// in its `\@maketitle` (else a dropped `\maketitle`'s) body — `\def`, `\gdef`, `\let`, `\newcommand`, `\providecommand`
/// — with a global one kept past its group, in source order; each with its definee.
pub(in crate::base_utilities) fn title_code_author_definitions() -> Vec<(Token, Vec<Token>)> {
  [T_CS!("\\@maketitle"), T_CS!("\\lx@dropped@maketitle")]
    .iter()
    .find_map(|cs| match lookup_meaning(cs) {
      Some(Stored::Expandable(definition)) => match definition.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => definitions_at_author(body.unlist_ref()),
        _ => None,
      },
      _ => None,
    })
    .unwrap_or_default()
}

/// The definition statements of `body`'s groups open at its first `\@author` ([`title_code_author_definitions`]),
/// in source order; `None` when it reads no `\@author`. Groups are brace groups, `\bgroup`/`\egroup`,
/// `\begingroup`/`\endgroup` and environments. A statement runs locally where it is replayed: a `\gdef` is a `\def`
/// there, and the `\global` before a definition is left out. `\edef`/`\xdef` are not replayed (expanded now, not at
/// the title code), nor `\renewcommand` (of a command already defined, which stands). A statement whose shape is not
/// read whole (a `\def` without its body's `{`, a `\newcommand` without a body) is left out, and so is one that closes
/// a group it did not open (2601.13359's `\def\affiliations{\egroup\par\Large\bgroup\rm}`, `\let\x\egroup`): the
/// title code's layout between its own groups, which the author content is not inside.
pub(in crate::base_utilities) fn definitions_at_author(
  body: &[Token],
) -> Option<Vec<(Token, Vec<Token>)>> {
  // the index after the balanced group starting at `i` (a BEGIN), else after the one token there
  let group_end = |i: usize| -> usize {
    if body.get(i).map(|t| t.get_catcode()) != Some(Catcode::BEGIN) {
      return (i + 1).min(body.len());
    }
    let mut depth = 0usize;
    for (j, t) in body.iter().enumerate().skip(i) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return j + 1;
          }
        },
        _ => {},
      }
    }
    body.len()
  };
  let skip_spaces = |mut j: usize| -> usize {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  // each statement with the group it is made in (None: global)
  let mut statements: Vec<(Token, Vec<Token>, Option<usize>)> = Vec::new();
  let mut open: Vec<usize> = vec![0];
  let mut groups = 0usize;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    let name = t.with_str(str::to_string);
    let opens = t.get_catcode() == Catcode::BEGIN
      || matches!(name.as_str(), "\\bgroup" | "\\begingroup" | "\\begin");
    let closes = t.get_catcode() == Catcode::END
      || matches!(name.as_str(), "\\egroup" | "\\endgroup" | "\\end");
    if opens {
      groups += 1;
      open.push(groups);
      i += 1;
      continue;
    }
    if closes {
      if open.len() > 1 {
        open.pop();
      }
      i += 1;
      continue;
    }
    if t == T_CS!("\\@author") {
      return Some(
        statements
          .into_iter()
          .filter(|(_, _, group)| group.is_none_or(|g| open.contains(&g)))
          .map(|(definee, statement, _)| (definee, statement))
          .collect(),
      );
    }
    let global = name == "\\gdef" || (i > 0 && body[i - 1] == T_CS!("\\global"));
    // the definee, where the statement ends, and its first token as replayed
    let read: Option<(Option<Token>, usize, Token)> = match name.as_str() {
      "\\def" | "\\gdef" if t.get_catcode() == Catcode::CS => {
        // the parameter text runs to the body's `{`
        let mut j = i + 2;
        while j < body.len() && body[j].get_catcode() != Catcode::BEGIN {
          j += 1;
        }
        (j < body.len()).then(|| (body.get(i + 1).copied(), group_end(j), T_CS!("\\def")))
      },
      "\\let" if t.get_catcode() == Catcode::CS => {
        let mut j = skip_spaces(i + 2);
        if body.get(j) == Some(&T_OTHER!("=")) {
          j += 1;
        }
        j = skip_spaces(j);
        body
          .get(j)
          .filter(|target| !matches!(target.get_catcode(), Catcode::BEGIN | Catcode::END))
          .map(|_| (body.get(i + 1).copied(), j + 1, t))
      },
      "\\newcommand" | "\\providecommand" if t.get_catcode() == Catcode::CS => {
        let mut j = i + 1;
        if body.get(j) == Some(&T_OTHER!("*")) {
          j += 1;
        }
        j = skip_spaces(j);
        let definee = match body.get(j) {
          Some(b) if b.get_catcode() == Catcode::BEGIN => body.get(j + 1).copied(),
          other => other.copied(),
        };
        j = skip_spaces(group_end(j));
        while body.get(j) == Some(&T_OTHER!("[")) {
          while j < body.len() && body[j] != T_OTHER!("]") {
            j += 1;
          }
          j = skip_spaces(j + 1);
        }
        body
          .get(j)
          .filter(|b| b.get_catcode() != Catcode::END)
          .map(|_| (definee, group_end(j), t))
      },
      _ => None,
    };
    match read {
      Some((Some(definee), stop, head))
        if matches!(definee.get_catcode(), Catcode::CS | Catcode::ACTIVE)
          && stop <= body.len()
          && closes_only_its_own_groups(&body[i + 2..stop]) =>
      {
        let mut statement = vec![head];
        statement.extend_from_slice(&body[i + 1..stop]);
        statements.push((
          definee,
          statement,
          (!global).then(|| *open.last().unwrap_or(&0)),
        ));
        i = stop;
      },
      _ => i += 1,
    }
  }
  None
}

/// Whether `tokens` (a definition's meaning) close only the groups and environments they open (`\bgroup`,
/// `\begingroup`, `\begin`): an author separator `\def\sep{\end{tabular}\begin{tabular}{c}}` does not.
fn closes_only_its_own_groups(tokens: &[Token]) -> bool {
  let mut depth = 0usize;
  for t in tokens {
    if t.get_catcode() != Catcode::CS {
      continue;
    }
    match t.with_str(str::to_string).as_str() {
      "\\bgroup" | "\\begingroup" | "\\begin" => depth += 1,
      "\\egroup" | "\\endgroup" | "\\end" => match depth.checked_sub(1) {
        Some(d) => depth = d,
        None => return false,
      },
      _ => {},
    }
  }
  true
}

/// The marks that request an affiliation label in an author (`\lx@author@withsup`, llncs `\inst`) and set it in an
/// affiliation (`\lx@affiliation@withsup`, `\lx@affiliation@withinst`): the author marks ([`is_author_mark`]), llncs's
/// `\inst` and the symbol mark.
pub(in crate::base_utilities) fn is_affiliation_mark(t: &Token) -> bool {
  is_author_mark(t) || *t == T_CS!("\\inst") || *t == T_CS!("\\lx@frontmatter@symbolmark")
}

/// The operand of each mark in `tokens`, in order, read as `read_frontmatter_sup_operand` reads it.
pub(in crate::base_utilities) fn mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  let mut operands = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if is_affiliation_mark(&tokens[i])
      && let Some((operand, next)) = sup_operand_at(tokens, i + 1)
    {
      operands.push(operand);
      i = next;
    } else {
      i += 1;
    }
  }
  operands
}

/// The marks an author's tokens request: those outside its notes, a `\thanks{...}` or `\footnote{...}` whose own
/// superscripts are its text or label it (`note_body`) — `Ann Able\thanks{$^{a}$ Supported by ...}` requests nothing.
pub(in crate::base_utilities) fn author_mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  mark_operands(&outside_notes(tokens))
}

/// An author's tokens without its `\thanks{...}` and `\footnote{...}` (and their optional label).
pub(in crate::base_utilities) fn outside_notes(tokens: &[Token]) -> Vec<Token> {
  let mut outside = Vec::with_capacity(tokens.len());
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\thanks") || tokens[i] == T_CS!("\\footnote") {
      let mut j = i + 1;
      if tokens.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[j..].iter().position(|t| *t == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if let Some((_, next)) = sup_operand_at(tokens, j) {
        i = next;
        continue;
      }
    }
    outside.push(tokens[i]);
    i += 1;
  }
  outside
}

/// The notes and requests an affiliation line under a name carries in a per-author class's `\author` (`Steward
/// Observatory\altaffilmark{2}`, `Univ A\thanks{Grant}`), taken off the line with their arguments to annotate the
/// author, as Perl, adding that `\author` whole, gives them to it: digested inside the affiliation, an `\altaffilmark`
/// request made a creator of its own and a `\thanks` a document-level note. The article `\author` nests the line as
/// Perl does (Base_Utility.pool.ltxml:723-725), so a `\thanks` there stays a document note (pubnote, as Perl's
/// `digestFrontmatterItem` binds it in a contact, :332-333; html_feedback#6888).
pub(in crate::base_utilities) fn take_author_annotations(line: &Tokens) -> (Tokens, Vec<Token>) {
  let notes = [
    T_CS!("\\thanks"),
    T_CS!("\\thanksref"),
    T_CS!("\\footnote"),
    T_CS!("\\altaffilmark"),
  ];
  let v = line.unlist_ref();
  let mut kept = Vec::with_capacity(v.len());
  let mut taken = Vec::new();
  let mut i = 0;
  while i < v.len() {
    if notes.contains(&v[i]) {
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = v[j..].iter().position(|t| *t == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if let Some((_, next)) = sup_operand_at(v, j) {
        taken.extend_from_slice(&v[i..next]);
        i = next;
        continue;
      }
    }
    kept.push(v[i]);
    i += 1;
  }
  (Tokens::new(kept), taken)
}

/// The operand of an affiliation entry's first mark, which `\lx@sup@setlabel@affiliation` labels it by (a later
/// superscript in the entry is its text, 63i).
pub(in crate::base_utilities) fn entry_mark(entry: &[Token]) -> Option<Vec<Token>> {
  mark_operands(entry).into_iter().next()
}

/// The label `\lx@sup@setlabel@affiliation` sets for a mark's operand: the first of its labels.
pub(in crate::base_utilities) fn mark_label(operand: &[Token]) -> Option<String> {
  clean_frontmatter_labels(&Tokens::new(operand.to_vec()).to_string(), "affiliation")
    .into_iter()
    .next()
}

/// The queue position of the first author of the row an address shared by a row of names belongs to
/// ([`shared_affiliation_calls`]): the authors queued after the last affiliation queued — on its own, or in an author's
/// entry (a paired row's `Ann Able\lx@add@affiliation{Univ A}`) —, those an `annotate=run` affiliation goes to; when
/// every queued author has one, the authors of the last author list (`\lx@add@authors@list`: 2010.08573's second
/// `\aistatsaddress`, the e-mails under a row its first address was paired with).
pub(in crate::base_utilities) fn queued_row_start() -> usize {
  let affiliation = T_CS!("\\lx@add@affiliation");
  let (authors, start) = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => {
      let (mut authors, mut start) = (0, 0);
      for (tag, attr, content) in queue {
        let role = attr.get("role").map(String::as_str);
        if tag == "ltx:creator" && role == Some("author") {
          authors += 1;
          if content.unlist_ref().contains(&affiliation) {
            start = authors;
          }
        } else if tag == "ltx:contact" && role == Some("affiliation") {
          start = authors;
        }
      }
      (authors, start)
    },
    _ => (0, 0),
  });
  if start < authors {
    return start;
  }
  match lookup_value("lx_authors_list_start") {
    Some(Stored::Int(list_start)) if (list_start as usize) < authors => list_start as usize,
    _ => start,
  }
}

/// The mark operands of each queued author, in queue order: the n-th digests with the label `author:n`
/// (`num_ltx:creator{author}`), by which an address goes to it — as the count starts from no author, which holds
/// unless authors an earlier flush digested are still kept (an appending `\author` after `\maketitle`).
pub(in crate::base_utilities) fn queued_author_marks() -> Vec<Vec<Vec<Token>>> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .map(|entry| author_mark_operands(entry.2.unlist_ref()))
      .collect(),
    _ => Vec::new(),
  })
}

/// Whether the addresses of `lines` may be the queued authors' in order from the one at `offset`
/// ([`addresses_in_order`]).
pub(in crate::base_utilities) fn run_in_author_order(
  lines: &[(Vec<Token>, Tokens)],
  offset: usize,
) -> bool {
  let keys = queued_author_name_keys();
  let addresses: Vec<String> = lines
    .iter()
    .flat_map(|(_, line)| email_addresses(line).unwrap_or_default())
    .collect();
  addresses_in_order(&keys, &addresses, offset)
}

/// Whether `addresses` may be the authors' in order from the one at `offset` (names keyed by `keys`, [`NameKeys`]):
/// none spells the name of an author other than the one at its place. 2509.10377's
/// `{12421181,ziyuzhao.cs,wu\_zhiliang,yangyics,…}@zju.edu.cn` lists one address for each author, by institution —
/// "yangyics" at Zhiliang Wu's place spells Yi Yang (2406.06326: "jyzhou" at Baolin Peng's, 63m).
pub(in crate::base_utilities) fn addresses_in_order(
  keys: &[NameKeys],
  addresses: &[String],
  offset: usize,
) -> bool {
  addresses
    .iter()
    .enumerate()
    .all(|(at, address)| spelled_owner(keys, address).is_none_or(|owner| owner == offset + at))
}

/// `tokens` without the marks that print nothing: a mark command's empty argument (`\textsuperscript{}`, [`mark_commands`])
/// and an empty math superscript (`$^{}$`), spaces and `\relax` aside, when the block holds no other mark. An empty mark is no mark
/// (user ruling R11 2026-10-20): it makes no block a marked one (2604.19440's `\textsuperscript{}Univ. Grenoble Alpes,
/// …` under four unmarked names, which pdflatex prints with no mark, an affiliation of all four). In a block other marks
/// make a marked one it stays, as before: it labels nothing there, and each name it ends is an author's piece of its
/// own (2604.22129's `Robert Maier\textsuperscript{}~~~Aljaz Bozic\textsuperscript{}` beside `David
/// Recasens\textsuperscript{\rm 1}`, which a marked block's marker-less pieces would weld into one creator). A note's
/// text is the note's, not the block's marks: what a `\thanks`, `\footnote` or `\footnotetext` (or a document's note
/// command, [`is_note_macro`]) holds is kept as written, so the note is read on what the author wrote — by the XML's
/// no-ink test as by the frontmatter observer's rehearsal of the same block (`\thanks{\textsuperscript{}}`: taken out,
/// the XML's note was empty and dropped where the observer's kept it; OXIDIZED_DESIGN_DIVERGENCES #916).
pub(in crate::base_utilities) fn drop_empty_marks(tokens: Tokens) -> Tokens {
  let original = tokens.clone();
  let v = tokens.unlist();
  // (a `\relax` in it prints nothing either: `Ann Able\textsuperscript{\relax}, Bob Baker`, the r7 re-review's n10)
  let blank = |range: &[Token]| {
    range.iter().all(|t| {
      matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::SPACE
      ) || *t == T_CS!("\\relax")
    })
  };
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    if NOTE_TEXT_COMMANDS.iter().any(|cs| v[i] == T_CS!(*cs)) || is_note_macro(&v[i]) {
      // (the command, an optional `[…]`, and its braced text, as written)
      let mut end = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      if v.get(end) == Some(&T_OTHER!("[")) {
        end = v[end..]
          .iter()
          .position(|t| *t == T_OTHER!("]"))
          .map_or(v.len(), |at| end + at + 1);
        end += v[end..].iter().take_while(|t| **t == T_SPACE!()).count();
      }
      let close = skip_group(&v, end).max(i + 1).min(v.len());
      out.extend_from_slice(&v[i..close]);
      i = close;
      continue;
    }
    let (operand, after) = if mark_commands().contains(&v[i]) {
      let open = i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      (open, 0)
    } else if v[i].get_catcode() == Catcode::MATH && v.get(i + 1) == Some(&T_SUPER!()) {
      (i + 2, 1)
    } else {
      out.push(v[i]);
      i += 1;
      continue;
    };
    let close = skip_group(&v, operand);
    if v
      .get(operand)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && blank(&v[operand..close.min(v.len())])
      && (after == 0
        || v
          .get(close)
          .is_some_and(|t| t.get_catcode() == Catcode::MATH))
    {
      i = close + after;
    } else {
      out.push(v[i]);
      i += 1;
    }
  }
  let out = Tokens::new(out);
  if has_author_marks(&rewrite_symbol_superscripts(out.clone())) {
    original
  } else {
    out
  }
}

/// The note commands whose braced argument is a note's text ([`drop_empty_marks`]).
const NOTE_TEXT_COMMANDS: &[&str] = &["\\thanks", "\\footnote", "\\footnotetext"];

/// The operand of a mark at `tokens[i..]`, as `read_frontmatter_sup_operand` reads it from the input — a group's
/// content, a control sequence with the group after it, else one token — and the index after it.
pub(in crate::base_utilities) fn sup_operand_at(
  tokens: &[Token],
  mut i: usize,
) -> Option<(Vec<Token>, usize)> {
  while tokens.get(i) == Some(&T_SPACE!()) {
    i += 1;
  }
  let group_end = |start: usize| {
    let mut depth = 0usize;
    for (j, t) in tokens.iter().enumerate().skip(start) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth = depth.saturating_sub(1);
          if depth == 0 {
            return Some(j);
          }
        },
        _ => {},
      }
    }
    None
  };
  let first = *tokens.get(i)?;
  if first.get_catcode() == Catcode::BEGIN {
    let end = group_end(i)?;
    return Some((tokens[i + 1..end].to_vec(), end + 1));
  }
  if first.get_catcode() == Catcode::CS
    && tokens
      .get(i + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let end = group_end(i + 1)?;
    return Some((tokens[i..=end].to_vec(), end + 1));
  }
  Some((vec![first], i + 1))
}
