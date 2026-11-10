//! The frontmatter API's Kind step for an affiliation-kind entry (`\lx@add@frontmatter`'s `kind_entry_addresses`): the
//! addresses an entry prints, each its own contact digested inside it, and the entry key the Link step reads. Moved
//! unchanged from `author_block/affiliations.rs` (Z18a): the API's own step, which every binding's contact goes through.
//! The e-mail line readers it shares with the bindings (`email_line_calls`, `line_addresses`, …) moved beside it, to
//! `email_lines.rs` and `affiliations.rs` (Z18j).

use super::*;

/// The Kind step for the addresses printed inside an affiliation-kind entry (FRONTMATTER_SINGLE_ENGINE_PLAN "The structure
/// model"; census127 cluster 1, the address family), at the frontmatter API's one call site after the Cut
/// (`\lx@annotate@frontmatter`, each piece of the Cut passing once): an affiliation, alternate affiliation or address
/// contact ([`CUT_ROLES`]) holding a line of e-mail addresses — a list (`\{aable, bbaker, ccole\}@a.edu`, `a@x, b@y`),
/// one behind an e-mail label (`Email: a@x, b@y`, its label the contacts' name), an e-mail command's (`\email{…}`), or
/// the rest of the entry's first line behind the mark that labels it (`{\tt $^1$\{aable, ccole\}@a.edu}`, the marked
/// entry the author block cut it as) — is that entry's content without those lines, each address an e-mail call digested
/// inside it ([`AddressPlacement::Inner`], `email_line_calls`): an e-mail of the entry's, whose owners the Link step
/// gives (`owners::entry_address_owners`, by the `_entry` key `\lx@annotate@frontmatter@now` gives the entry and
/// its inner contacts). An entry of addresses alone stays as their carrier, empty, and is dropped once they are placed
/// (`insert::drop_empty_contacts`). `None` for any other contact, or one holding no such line. The lines are read by the
/// author block's reader of an address line ([`line_addresses`]): an address inside a line of other words (`Univ X,
/// Town; ann@x.org`) is the frontmatter observer's text reading's alone (`Resolver::take_out_entry_addresses`), and a
/// line these tokens do not show — a document's macro, `\authorcr`, a connective "and" — the flush's Kind step's, which
/// reads it as printed (`insert::kind_address_lines`, OD #966; 2608.22602, 2111.01631, 2302.02638). A
/// template (`\{firstname.lastname\}@x.org`, [`is_pattern_address`]) or a list the reader could not part
/// ([`lists_local_parts`]) stays where it prints. Witnesses: authblk's `\affil{… \\ \{…\}@x}` (2502.01754) and
/// `\affil[ ]{\texttt{\{…\}@x}}` (2602.21641, 2307.02047), the kernel's `{\tt $^1$\{…\}@x}` lines (1901.09107), llncs's
/// `\email{\{ivan|carolin\}@x}` (0902.3918, 1909.00276), aaspp4's `$^3$…;\\ Email: dzg@, xywang@, hyf@`
/// (astro-ph0512555).
pub(in crate::base_utilities) fn kind_entry_addresses(
  tag: &Tokens,
  kv: Option<&KeyVals>,
  content: &Tokens,
) -> Result<Option<Tokens>> {
  if tag.to_string() != "ltx:contact"
    || !content
      .unlist_ref()
      .iter()
      .any(|t| t.with_str(|s| s == "@"))
  {
    return Ok(None);
  }
  let role = kv
    .and_then(|kv| kv.get_value("role"))
    .map(ToString::to_string)
    .unwrap_or_default();
  // (an entry a class shares among every author, llncs's one `\institute`, `_share=authors`: its lines of addresses are
  // the shared row's, placed by the same chain where the row is read — `shared_row::row_email_calls`, OD #901;
  // 1103.2027)
  if !CUT_ROLES.contains(&role.as_str()) || kv.is_some_and(|kv| kv.get_value("_share").is_some()) {
    return Ok(None);
  }
  let (withinst, inner) = within_inst(content);
  let (withsup, inner) = within_wrapper(&inner, "\\lx@affiliation@withsup");
  // (a footnote of the entry's holding addresses alone, behind a label or none — REVTeX's `\affiliation{Univ A,
  // Town\footnote{Electronic address: aable@a.edu}}`, the note the class's own `\email` prints (revtex4-2.cls:2404):
  // its addresses the entry's, as an address line's; Perl keeps the footnote a `<pubnote>` of the document,
  // Base_Utility.pool.ltxml:331 — 1002.3223, cond-mat0202268)
  let (inner, footnotes) = take_address_footnotes(inner);
  // (a line the reader takes out: its addresses read, none a template or a list it could not part, and its groups
  // balanced, so the entry left is)
  // (a line of e-mail commands is e-mails already, the class's own contacts with the label it prints — llncs's
  // `\email{…}` "E-mail", 2008.05861's `\email{\{jlwang,yhliu\}@…}, \email{jianbo@…}` —, digested inside the entry
  // and so its addresses: left as it is)
  // (a line holding any other command — an ORCID, a note, a symbol — is no line of addresses alone: it stays in the
  // entry as it prints, its addresses the text reading's, [`prints_addresses_only`]; 2308.02269's
  // `\url{{arim,sue}@ist.hokudai.ac.jp}\orcidID{0000-0002-2701-0271}` had read the ORCID's digits into the domain and
  // lost the ORCID, review rev100b B1)
  let addresses_of = |line: &Tokens| {
    let command = line
      .unlist_ref()
      .iter()
      .any(|t| t.get_catcode() == Catcode::CS && is_email_wrapper(t));
    (!command && prints_addresses_only(line))
      .then(|| line_addresses(line))
      .flatten()
      .filter(|addresses| {
        balanced(line.unlist_ref())
          && addresses
            .iter()
            .all(|a| !is_pattern_address(a) && !lists_local_parts(a))
      })
  };
  let mut kept: Vec<(Vec<Token>, Vec<Token>)> = Vec::new();
  // (each address line, and for a mailbox link line the label before its link: [`mailbox_link_line`])
  let mut address_lines: Vec<(Tokens, Option<Option<Tokens>>)> = Vec::new();
  // (for each address line, whether no line the entry keeps prints before it)
  let mut leading: Vec<bool> = Vec::new();
  let mut first = true;
  // (whether the entry prints anything but addresses, its mark aside)
  let mut prints_else = false;
  for (delimiter, line) in split_tokens_delimited(inner, affil_splits()) {
    let printed = !line.is_empty() && !prints_nothing(&line);
    // (the first line of a marked entry: the mark that labels it kept, the addresses behind it the entry's)
    if printed
      && first
      && withsup
      && let Some((mark, rest)) = mark_led_addresses(&line)
      && addresses_of(&rest).is_some()
    {
      kept.push((delimiter, mark));
      address_lines.push((rest, None));
      leading.push(false);
      first = false;
      continue;
    }
    // (a line a mark leads is no address line: its mark is no letter of an address, `$^2$bob@y.org`)
    if printed && author_mark_position(&line).is_none() && addresses_of(&line).is_some() {
      address_lines.push((line, None));
      leading.push(!prints_else);
      first = false;
      continue;
    }
    if printed
      && author_mark_position(&line).is_none()
      && let Some((label, link)) = mailbox_link_line(&line)
    {
      address_lines.push((link, Some(label)));
      leading.push(!prints_else);
      first = false;
      continue;
    }
    first &= !printed;
    prints_else |= printed;
    kept.push((delimiter, line.unlist()));
  }
  if address_lines.is_empty() && footnotes.is_empty() {
    return Ok(None);
  }
  let mut entry: Vec<Token> = Vec::new();
  for (delimiter, line) in kept {
    if !entry.is_empty() {
      entry.extend(delimiter);
    }
    entry.extend(line);
  }
  let mut entry = trim_trailing_separator(entry);
  // (an entry of addresses alone that the class names, `\mail{…}`'s "Correspondence to", REVTeX's
  // `\altaffiliation[E-mail address: ]{…}`: its name the e-mails' — where no label of their own names them; 0709.0811,
  // astro-ph0108379, cond-mat0703234, 1402.3092)
  let entry_name: Option<Vec<Token>> = (!prints_else && footnotes.is_empty())
    .then(|| kv.and_then(|kv| kv.revert().ok()))
    .flatten()
    .and_then(|attrs| {
      split_tokens(attrs, vec![T_OTHER!(",").into()])
        .into_iter()
        .find(|item| item.to_string().trim_start().starts_with("name="))
        .map(Tokens::unlist)
    })
    // (not the role's own default, `\lx@contact@altaffiliation@name` — "Alternate Affiliation:" names no e-mail, 0812.4856
    // —, nor an empty one, which names nothing: aa's `\mail` copy, 97b)
    .filter(|name| {
      let value = name.iter().skip_while(|t| **t != T_OTHER!("=")).skip(1);
      !name
        .iter()
        .any(|t| t.get_catcode() == Catcode::CS && t.with_str(|s| s.starts_with("\\lx@contact@")))
        && value.clone().any(|t| {
          matches!(
            t.get_catcode(),
            Catcode::LETTER | Catcode::OTHER | Catcode::CS
          )
        })
    });
  // (each e-mail the Kind step made says so, `ENTRY_KINDED`: an e-mail command the author wrote inside the entry is
  // placed by its own call where it holds one address, `insert::link_entry_addresses`)
  let kinded = mouth::tokenize_internal(TeXString::assembled(s!("{ENTRY_KINDED}=true"))).unlist();
  // (a line printed before every line the entry keeps, its e-mails placed before the entry: user ruling R8,
  // `\IEEEauthorblockA{\href{mailto:ann@a.org}{ann@a.org}\\ \url{https://orcid.org/…}\\ …}`, 2206.10577 — review
  // rev107c N1, `insert::LINE_FIRST`)
  let kinded_first = mouth::tokenize_internal(TeXString::assembled(s!(
    "{ENTRY_KINDED}=true,{LINE_FIRST}=true"
  )))
  .unlist();
  for ((line, link_label), leading) in address_lines.into_iter().zip(leading) {
    let kinded = if leading && prints_else {
      &kinded_first
    } else {
      &kinded
    };
    // (the line's own label — a mailbox link line's text before its link, else its e-mail label — names its contacts,
    // after the class's name of the entry where both print, one rule: [`entry_line_label`]; else the class's name)
    let labelled = match link_label {
      Some(label) => label.map(|label| (label, line.clone())),
      None => email_line_label(&line),
    };
    let calls = match (labelled, &entry_name) {
      (Some((label, rest)), name) => named_email_calls(
        email_line_calls(rest, AddressPlacement::Inner)?,
        &entry_line_label(name.as_deref(), label),
      ),
      (None, Some(name)) => {
        optioned_email_calls(email_line_calls(line, AddressPlacement::Inner)?, name)
      },
      (None, None) => email_line_calls(line, AddressPlacement::Inner)?,
    };
    entry.extend(optioned_email_calls(calls, kinded));
  }
  for (label, line) in footnotes {
    let calls = email_line_calls(line, AddressPlacement::Inner)?;
    let calls = match label {
      Some(label) => named_email_calls(calls, &label),
      None => calls,
    };
    entry.extend(optioned_email_calls(calls, &kinded));
  }
  let mut entry = Tokens::new(entry);
  if withsup {
    entry = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(entry)]);
  }
  if withinst {
    entry = Invocation!(T_CS!("\\lx@affiliation@withinst"), vec![Some(entry)]);
  }
  DebugFeature!(
    "frontmatter",
    "FRONT kind {role}: its address lines the entry's e-mails: {entry}"
  );
  Ok(Some(entry))
}

/// The state key naming the affiliation-kind entry whose content is being digested (`<key>:symbol` or `<key>:`), whose
/// e-mails take its [`ENTRY_KEY`] (and its [`ENTRY_SYMBOL`]); empty outside one.
pub(in crate::base_utilities) const ENTRY_DIGESTING: &str = "lx_frontmatter_entry_digesting";

/// The state key counting the affiliation-kind entries given an [`ENTRY_KEY`].
pub(in crate::base_utilities) const ENTRY_KEY_COUNTER: &str = "lx_frontmatter_entry_key";

/// Whether a footnote symbol leads an entry's content (in the affiliation's own wrappers): a text symbol (`* Equal
/// contribution`, `\dag …`, [`starts_with_footnote_symbol`]) or a superscript one (`$^\dagger$Univ X`).
pub(in crate::base_utilities) fn led_by_footnote_symbol(content: &Tokens) -> bool {
  let (_, inner) = within_inst(content);
  let (_, inner) = within_wrapper(&inner, "\\lx@affiliation@withsup");
  starts_with_footnote_symbol(&inner)
    || (marker_leads(&inner)
      && entry_mark(inner.unlist_ref()).is_some_and(|mark| is_symbol_mark(&mark)))
}

/// `content` without the notes (`\footnote{…}`, `\thanks{…}`, an optional mark aside) that print a line of addresses
/// alone, behind a label or none, and those lines, each with the label its contacts are named by when the line's own
/// reader does not read it ([`note_addresses`]; [`kind_entry_addresses`]).
fn take_address_footnotes(content: Tokens) -> (Tokens, Vec<(Option<Tokens>, Tokens)>) {
  let v = content.unlist_ref();
  let note_cs = [T_CS!("\\footnote"), T_CS!("\\thanks")];
  if !v.iter().any(|t| note_cs.contains(t)) {
    return (content, Vec::new());
  }
  let mut kept: Vec<Token> = Vec::with_capacity(v.len());
  let mut lines: Vec<(Option<Tokens>, Tokens)> = Vec::new();
  let mut i = 0;
  while i < v.len() {
    if note_cs.contains(&v[i]) {
      let mut open = i + 1;
      while v.get(open) == Some(&T_SPACE!()) {
        open += 1;
      }
      // (its optional mark, `\footnote[2]{…}`)
      if v.get(open) == Some(&T_OTHER!("["))
        && let Some(close) = v[open..].iter().position(|t| *t == T_OTHER!("]"))
      {
        open += close + 1;
        while v.get(open) == Some(&T_SPACE!()) {
          open += 1;
        }
      }
      if v
        .get(open)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      {
        let end = skip_group(v, open);
        if end <= v.len()
          && end >= open + 2
          && let Some(line) = note_addresses(&Tokens::new(v[open + 1..end - 1].to_vec()))
        {
          lines.push(line);
          i = end;
          continue;
        }
      }
    }
    kept.push(v[i]);
    i += 1;
  }
  (Tokens::new(kept), lines)
}

/// The addresses line a note prints when it prints addresses alone: one [`line_addresses`] reads (a list, one behind
/// an e-mail label), or one behind a label ending in an e-mail label's words before its colon (`Corresponding authors
/// e-mail: cecilia.pennetta@unile.it`, cond-mat0202268; REVTeX's `Electronic address: …`, 1002.3223) — the label then
/// given apart, the contacts' name. None a template or a list the reader could not part.
fn note_addresses(note: &Tokens) -> Option<(Option<Tokens>, Tokens)> {
  let real = |addresses: Vec<String>| {
    addresses
      .iter()
      .all(|a| !is_pattern_address(a) && !lists_local_parts(a))
  };
  if line_addresses(note).is_some_and(real) {
    return Some((None, note.clone()));
  }
  let v = note.unlist_ref();
  let colon = v.iter().position(|t| *t == T_OTHER!(":"))?;
  let label: Vec<Token> = v[..colon]
    .iter()
    .copied()
    .skip_while(|t| *t == T_SPACE!())
    .collect();
  let text = visible_name_text(&label);
  let words: Vec<&str> = text.split_whitespace().collect();
  // (a label of a few words — `Corresponding authors e-mail`; a sentence before its colon is the note's text, `X. Zhang
  // and L. Duan are with …, Singapore. E-mail`, 1710.05616)
  let labelled = words.len() <= 4
    && (1..=2.min(words.len())).any(|n| is_email_label(&words[words.len() - n..].join(" ")));
  let rest = Tokens::new(
    v[colon + 1..]
      .iter()
      .copied()
      .skip_while(is_spacing)
      .collect(),
  );
  (labelled && email_addresses(&rest).is_some_and(real)).then(|| {
    let mut label = label;
    label.push(T_OTHER!(":"));
    (Some(Tokens::new(label)), rest)
  })
}

/// Whether `tokens` closes every group it opens, none before it opens it.
fn balanced(tokens: &[Token]) -> bool {
  let mut depth = 0usize;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => return false,
      Catcode::END => depth -= 1,
      _ => {},
    }
  }
  depth == 0
}

/// The commands a line of addresses alone prints them with, beside the font and size commands
/// ([`NAME_TEXT_COMMANDS`], [`FONT_SWITCH`]) and the spacing ([`is_spacing`], [`is_spacing_command`],
/// [`is_line_separator`]): the url and link commands, the printed braces, underscore and bar of a list of local parts
/// ([`email_addresses`]), and the boxes and no-ops round them.
const ADDRESS_LINE_COMMANDS: &[&str] = &[
  "\\url",
  "\\href",
  "\\nolinkurl",
  "\\path",
  "\\{",
  "\\}",
  "\\_",
  "\\lbrace",
  "\\rbrace",
  "\\textbraceleft",
  "\\textbraceright",
  "\\textunderscore",
  "\\mid",
  "\\vert",
  "\\mathtt",
  "\\texttt",
  // (a text command printing its argument: `\text{sgadgil@stanford.edu}`, 2011.11850; `\verb"ndr@…"`, astro-ph0002160)
  "\\text",
  "\\verb",
  "\\protect",
  "\\relax",
  "\\mbox",
  "\\hbox",
  "\\par",
  "\\noindent",
  "\\quad",
  "\\smallskip",
  "\\medskip",
  "\\bigskip",
];

/// Whether `line` holds no command but those a line of addresses alone prints them with ([`ADDRESS_LINE_COMMANDS`], the
/// font, size and spacing commands; a command whose arguments print no letter, `\color{…}`, `\hspace{…}`, aside:
/// [`without_non_printing_arguments`]). Any other — `\orcidID{…}` (2308.02269), `\thanks{…}`, `$^\dagger$` — prints
/// more than the addresses, and the line stays in its entry as it prints (review rev100b B1).
fn prints_addresses_only(line: &Tokens) -> bool {
  without_non_printing_arguments(line.unlist_ref())
    .iter()
    .all(|t| {
      t.get_catcode() != Catcode::CS
        || is_spacing(t)
        || is_spacing_command(t)
        || is_line_separator(t)
        || t.with_str(|s| {
          NAME_TEXT_COMMANDS.contains(&s)
            || FONT_SWITCH.is_match(s)
            || ADDRESS_LINE_COMMANDS.contains(&s)
        })
    })
}

/// A line led by a printed mark (`$^1$…`, `\textsuperscript{1}…`, the font switches before it kept: `{\tt $^1$\{aable,
/// ccole\}@a.edu}`, a group round the whole line read inside), parted into the mark and what follows it, each in the
/// line's group: `None` for a line no printed mark leads (`relocate::leading_line_label`; a number typed by hand is no
/// mark here).
fn mark_led_addresses(line: &Tokens) -> Option<(Vec<Token>, Tokens)> {
  let v = line.unlist_ref();
  let grouped = v.len() >= 2 && v[0].get_catcode() == Catcode::BEGIN && skip_group(v, 0) == v.len();
  let inner = if grouped { &v[1..v.len() - 1] } else { v };
  let (_, typed, rest) = leading_line_label(&Tokens::new(inner.to_vec()))?;
  if typed {
    return None;
  }
  // (the rest is the switches before the mark and the tokens after it, those a tail of the line)
  let lead = inner
    .iter()
    .take_while(|t| {
      **t == T_SPACE!()
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| MARK_SWITCHES.contains(&s)))
    })
    .count();
  let tail = &rest.unlist_ref()[lead.min(rest.unlist_ref().len())..];
  let mark = &inner[..inner.len().checked_sub(tail.len())?];
  let wrap = |tokens: Vec<Token>| {
    if grouped {
      let mut out = vec![v[0]];
      out.extend(tokens);
      out.push(v[v.len() - 1]);
      out
    } else {
      tokens
    }
  };
  let mut address = inner[..lead].to_vec();
  address.extend_from_slice(tail);
  Some((wrap(mark.to_vec()), Tokens::new(wrap(address))))
}

/// The content of `content` inside the one-argument wrapper `cs` round all of it (`\lx@affiliation@withsup{…}`), and
/// whether it was.
fn within_wrapper(content: &Tokens, cs: &str) -> (bool, Tokens) {
  let v = content.unlist_ref();
  if v.len() >= 3
    && v[0] == T_CS!(cs)
    && v[1].get_catcode() == Catcode::BEGIN
    && skip_group(v, 1) == v.len()
  {
    (true, Tokens::new(v[2..v.len() - 1].to_vec()))
  } else {
    (false, content.clone())
  }
}

/// The content of a contact inside the affiliation's own wrapper (`\lx@affiliation@withinst{…}`, which
/// `\lx@add@affiliation` puts around it), and whether it was in it.
pub(super) fn within_inst(content: &Tokens) -> (bool, Tokens) {
  let within = content.unlist_ref();
  let withinst = within.len() >= 3
    && within[0] == T_CS!("\\lx@affiliation@withinst")
    && within[1].get_catcode() == Catcode::BEGIN
    && skip_group(within, 1) == within.len();
  if withinst {
    (true, Tokens::new(within[2..within.len() - 1].to_vec()))
  } else {
    (false, content.clone())
  }
}
