//! The frontmatter API's e-mail line readers, which its Kind step (`kind_entry_addresses`) and entry cutter
//! (`entry_calls`) share: the `\lx@add@email` calls for a line of addresses, its notes and font, a row's start and its
//! addresses in author order. Moved unchanged from `author_block/email_lines.rs` and `author_block/email_block_calls.rs`
//! (Z18j).

use super::*;

/// The `\lx@add@email` calls for a line of addresses in an author block (`a@x, b@y`, `{a,b}@dom`), placed as
/// `placement` says: one call per address, a whole-line wrapper repeated on each, or the line itself when it holds one
/// address (`\href{mailto:a@x}{a@x}` stays a link); else the line as one contact. OXIDIZED_DESIGN #52(j). A line led
/// by an e-mail label (`email: a@x, b@y`, `E-mails: …`) gives each contact that label as its name (label text is
/// markup, never dropped: FIDELITY_CHARTER §1; OXIDIZED_DESIGN_DIVERGENCES #954, astro-ph0405186).
pub(in crate::base_utilities) fn email_line_calls(
  line: Tokens,
  placement: AddressPlacement,
) -> Result<Vec<Token>> {
  let line = spacing_formulas_as_space(line);
  let Some((label, rest)) = email_line_label(&line) else {
    return unlabelled_email_line_calls(line, placement);
  };
  let calls = unlabelled_email_line_calls(rest, placement)?;
  Ok(named_email_calls(calls, &label))
}

/// Whether a formula's tokens, between its shifts, print spacing alone: spaces, ties and the space commands
/// ([`leading_spacing_end`]: `$~\quad~~$`, `$\,$`, `$\hspace{1em}$`), no glyph — latex.ltx:9411 `\nobreakspace` and
/// :9435 `\quad` are glue. A tie counts as read either way: active as written, other as a Semiverbatim argument holds
/// it (`\lx@add@email`, Perl's `neutralize`). The empty formula (`$$`) is none. Read on the tokens: the line readers
/// and the contact's reading ([`spacing_formulas_as_space`], `math_glyphs_as_printed`) run before the formula is
/// digested.
fn formula_prints_spacing_alone(inner: &[Token]) -> bool {
  let mut at = 0;
  while at < inner.len() {
    let end = match leading_spacing_end(&inner[at..]) {
      0 if inner[at] == T_OTHER!("~") => 1,
      0 => return false,
      end => end,
    };
    at += end;
  }
  !inner.is_empty()
}

/// `line` with each formula that prints spacing alone ([`formula_prints_spacing_alone`]) and the spaces around it one
/// space between the text around it, none at an edge of the line or of a group (`\lx@address@urlfont{$\,$a@x}`):
/// glue, as it prints, no text of an address, nor a row's separator inside its shifts. The shifts — math shifts, or
/// characters as a Semiverbatim argument holds them — pair left to right, as TeX pairs them, and every other formula
/// stays as written. `$~\quad~~$ cbray@mines.edu` had been read whole as the address, its source text the contact's
/// (`$˜ ˜˜$ cbray@mines.edu`, 2205.14113); authblk's `a@x $\quad$ b@y` parted at the `\quad` inside the shifts (`a@x
/// $`, `$ b@y`, 2302.05182). The line readers (`email_line_calls`, the Kind step) and the contact's reading
/// (`math_glyphs_as_printed`, any binding's `\email{$\,$a@x}`) read the line through it.
/// OXIDIZED_DESIGN_DIVERGENCES #972.
pub(in crate::base_utilities) fn spacing_formulas_as_space(line: Tokens) -> Tokens {
  let v = line.unlist_ref();
  let shift = |t: &Token| *t == T_OTHER!("$") || t.get_catcode() == Catcode::MATH;
  if !v.iter().any(shift) {
    return line;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut spaced = false;
  let mut at = 0;
  while at < v.len() {
    if shift(&v[at])
      && let Some(len) = v[at + 1..].iter().position(shift)
    {
      let pair = &v[at..at + len + 2];
      at += len + 2;
      if !formula_prints_spacing_alone(&pair[1..=len]) {
        out.extend_from_slice(pair);
        continue;
      }
      spaced = true;
      while out.last() == Some(&T_SPACE!()) {
        out.pop();
      }
      while v.get(at) == Some(&T_SPACE!()) {
        at += 1;
      }
      let text_before = out
        .last()
        .is_some_and(|t| t.get_catcode() != Catcode::BEGIN);
      let text_after = v.get(at).is_some_and(|t| t.get_catcode() != Catcode::END);
      if text_before && text_after {
        out.push(T_SPACE!());
      }
      continue;
    }
    out.push(v[at]);
    at += 1;
  }
  if spaced { Tokens::new(out) } else { line }
}

/// The `\lx@add@email` calls `calls`, each named by `label` (`name={<label>~}`), as an e-mail line's label names its
/// contacts ([`email_line_calls`]).
pub(in crate::base_utilities) fn named_email_calls(
  calls: Vec<Token>,
  label: &Tokens,
) -> Vec<Token> {
  let mut name: Vec<Token> = mouth::tokenize_internal("name=").unlist();
  name.push(T_BEGIN!());
  name.extend(label.unlist_ref().iter().copied());
  name.push(T_ACTIVE!('~'));
  name.push(T_END!());
  optioned_email_calls(calls, &name)
}

/// The `\lx@add@email` calls `calls`, each given the key-value `option` (`name={…}`) first among its options.
pub(in crate::base_utilities) fn optioned_email_calls(
  calls: Vec<Token>,
  option: &[Token],
) -> Vec<Token> {
  let mut out = Vec::with_capacity(calls.len());
  let mut i = 0;
  while i < calls.len() {
    out.push(calls[i]);
    if calls[i] == T_CS!("\\lx@add@email") {
      if calls.get(i + 1) == Some(&T_OTHER!("[")) {
        out.push(T_OTHER!("["));
        out.extend(option.iter().copied());
        out.push(T_OTHER!(","));
        i += 1;
      } else {
        out.push(T_OTHER!("["));
        out.extend(option.iter().copied());
        out.push(T_OTHER!("]"));
      }
    }
    i += 1;
  }
  out
}

/// [`email_line_calls`] of a line no e-mail label leads.
fn unlabelled_email_line_calls(line: Tokens, placement: AddressPlacement) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  // (an entry's line written out address by address: each in the font the line printed it in, [`in_line_font`])
  let inner = matches!(placement, AddressPlacement::Inner);
  // A wrapper is kept on each address (`\texttt`, `\small`, `\url`), but an email command's (`\email`, `\mailto`,
  // `\emailaddr`) is the contact `\lx@add@email` stands for — nested, it made a second (empty) contact and advanced
  // the `labelseq` count twice, so the second address found no author (llncs `\institute{… \\ \email{a@x, b@y}}`).
  // (the address set as the command sets it, [`email_command_address`])
  let unwrapped = |line: Tokens| match whole_line_cs_wrapper(&line) {
    Some((cmd, inner)) if is_email_wrapper(&cmd) => email_command_address(&cmd, inner),
    _ => Ok(line),
  };
  let addrs = email_addresses(&line).unwrap_or_default();
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
    AddressPlacement::Settled(_) => AddressPlacement::Shared,
    placement => placement,
  };
  let options: Vec<Option<Tokens>> = match placement {
    AddressPlacement::Inner if !addrs.is_empty() => addrs.iter().map(|_| None).collect(),
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
    let address = if inner {
      in_line_font(&line, address)
    } else {
      address
    };
    calls.extend(Invocation!(T_CS!("\\lx@add@email"), vec![opts, Some(address)]).unlist());
  }
  Ok(calls)
}

/// `address`, one of the addresses `line` prints, in the font `line` printed it in: the font and size declarations
/// ([`name_text_command`]) and wrappers (`\texttt{…}`, [`FONT_WRAPPERS`]) around every `@` the line prints, when they
/// are one — `{\tt \{able, baker\}@x.org}` (1804.05417), `$^\ddag$\tt\{kyding,ycwang12,…\}@iflytek.com` (2311.01166),
/// `\texttt{\{czhang, htwai, manchoso\}@se.cuhk.edu.hk}, \texttt{shangyuanliu@…}` (2410.24095) —, each written out in a
/// group of its own: `{\tt able@x.org}`, as a line printing one address is digested whole (review rev100b2 #2). A line
/// whose `@`s print in different fonts keeps `address` as it is; a whole-line wrapper ([`address_tokens`] repeats it)
/// is read inside.
fn in_line_font(line: &Tokens, address: Tokens) -> Tokens {
  // (a whole-line wrapper [`address_tokens`] repeats on each address already: the font inside it read —
  // `\small{\texttt{\{g.marra,alessandro.betti\}@unifi.it}}`, 2002.07720)
  let inner = match whole_line_cs_wrapper(line) {
    Some((cmd, _)) if is_email_wrapper(&cmd) => return address,
    Some((_, inner)) => inner,
    None => line.clone(),
  };
  let v = inner.unlist_ref();
  // (each group's font: the declarations and wrappers in force, in order — a wrapper `\texttt` followed by the group
  // it sets)
  let mut frames: Vec<Vec<(Token, bool)>> = vec![Vec::new()];
  let mut wrapper: Option<Token> = None;
  let mut at_signs: Vec<Vec<(Token, bool)>> = Vec::new();
  for t in v {
    match t.get_catcode() {
      Catcode::BEGIN => {
        let mut font = frames.last().cloned().unwrap_or_default();
        font.extend(wrapper.take().map(|w| (w, true)));
        frames.push(font);
      },
      Catcode::END => {
        wrapper = None;
        if frames.len() > 1 {
          frames.pop();
        }
      },
      Catcode::CS if t.with_str(|s| FONT_WRAPPERS.contains(&s)) => wrapper = Some(*t),
      Catcode::CS if name_text_command(t) => {
        wrapper = None;
        // (a declaration repeated before the next address is in force once: `{\tt\small a@x, \tt\small
        // \{b,c\}@y}`, 2311.04698)
        if let Some(font) = frames.last_mut()
          && !font.contains(&(*t, false))
        {
          font.push((*t, false));
        }
      },
      Catcode::SPACE => {},
      _ => {
        wrapper = None;
        if t.with_str(|s| s == "@") {
          at_signs.push(frames.last().cloned().unwrap_or_default());
        }
      },
    }
  }
  let Some(font) = at_signs
    .first()
    .filter(|font| !font.is_empty() && at_signs.iter().all(|other| other == *font))
  else {
    return address;
  };
  let mut out = vec![T_BEGIN!()];
  for (cmd, wraps) in font {
    out.push(*cmd);
    if *wraps {
      out.push(T_BEGIN!());
    }
  }
  out.extend(address.unlist());
  out.extend(font.iter().filter(|(_, wraps)| *wraps).map(|_| T_END!()));
  out.push(T_END!());
  Tokens::new(out)
}

/// The font commands taking the text they set as their argument ([`in_line_font`]).
const FONT_WRAPPERS: &[&str] = &[
  "\\texttt",
  "\\textbf",
  "\\textit",
  "\\textsc",
  "\\textsl",
  "\\textup",
  "\\textmd",
  "\\textrm",
  "\\textsf",
  "\\textnormal",
  "\\emph",
];

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

/// An address line carrying notes (`\texttt{a@x, b@y}\thanks{Submitted …}`, 2605.23553; 2304.05485's copyright
/// `\thanks`, 1710.01254, 1803.07937, 2603.09888, 2608.24641) is still an address line: the line without its notes
/// ([`take_author_annotations`]) and the blank lines and spaces it then ends with, and the notes as the document's ([`document_note_call`]), when the rest is an address
/// line by the caller's reader (`is_address_line`) and every note taken is a plain `\thanks{…}` or `\footnote{…}` — as
/// they digested in the contact the line made before (a pubnote), now after the block's calls. Read whole, the note's
/// text is no address (`email_addresses` reads the printed text), so the line was an affiliation: the last name's, or
/// every name's. `None` for a line with no note, a rest that is no address line, or a note of another kind
/// (`\thanksref`, `\altaffilmark`, `\thanks[…]{…}`), each line read as before. `email_addresses` itself keeps reading
/// the note: [`address_tokens`] would drop a note it skipped.
pub(in crate::base_utilities) fn address_line_notes(
  line: &Tokens,
  is_address_line: impl Fn(&Tokens) -> Result<bool>,
) -> Result<Option<(Tokens, Vec<Token>)>> {
  if !line
    .unlist_ref()
    .iter()
    .any(|t| *t == T_CS!("\\thanks") || *t == T_CS!("\\footnote"))
  {
    return Ok(None);
  }
  let (rest, taken) = take_author_annotations(line);
  // (without the blank lines and spaces before the notes, which print nothing there: the address a line's wrapper
  // holds whole, its font kept — 2605.23553's `\texttt{…}<blank lines>\thanks{…}`)
  let mut rest = rest.unlist();
  while rest
    .last()
    .is_some_and(|t| *t == T_SPACE!() || *t == T_CS!("\\par"))
  {
    rest.pop();
  }
  let rest = Tokens::new(rest);
  if taken.is_empty() || !is_address_line(&rest)? {
    return Ok(None);
  }
  let Some(spans) = line_notes(&taken) else {
    return Ok(None);
  };
  let mut notes = Vec::with_capacity(taken.len());
  for (start, end) in spans {
    let Some(call) = document_note_call(&taken[start..end]) else {
      return Ok(None);
    };
    notes.extend(call);
  }
  Ok(Some((rest, notes)))
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

/// Whether the addresses of `lines` may be the queued authors' in order from the one at `offset`
/// ([`addresses_in_order`]).
pub(in crate::base_utilities) fn run_in_author_order(
  lines: &[(Vec<Token>, Tokens)],
  offset: usize,
) -> Result<bool> {
  let keys = queued_author_name_keys()?;
  let addresses: Vec<String> = lines
    .iter()
    .flat_map(|(_, line)| line_addresses(line).unwrap_or_default())
    .collect();
  Ok(addresses_in_order(&keys, &addresses, offset))
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

/// A marker-less line that is *purely* a list of email addresses — a shared
/// `\texttt{a@x, b@y,}` / `\email{...}` / `\{a,b\}@x` line covering all authors. Like
/// `line_is_email` but tolerates a comma-separated list (and the trailing
/// comma authors often leave), as `email_addresses` reads it: every item a full
/// address or a braced list of local parts before its domain. A prose affiliation
/// line ("Dept. of Foo, University of Pisa, Italy") is rejected, so this never
/// misclassifies an address as an email.
pub fn line_is_email_list(line: &Tokens) -> bool { email_addresses(line).is_some() }
