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

/// The phrases by which a note or a line names the corresponding author ([`names_correspondence`]).
static CORRESPONDENCE_PHRASE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
  Regex::new(concat!(
    r"corresponding[\s-]+authors?\b",
    r"|correspondence\s*(?::|to\b|should\b|may\b|author|and\s+requests\b)",
    r"|\bfor\s+correspondence\b",
    r"|\b(?:whom|any|all)\s+(?:\w+\s+)?correspondence\b"
  ))
  .unwrap()
});

/// Whether a note's text names its author the corresponding one, by a phrase that says so ([`CORRESPONDENCE_PHRASE`]:
/// "Corresponding author.", "Corresponding authors", "Correspondence to …", "Correspondence: …", "to whom (any)
/// correspondence should be addressed", "Author for correspondence"), not by the word alone: a "corresponding member"
/// of an academy, a note on "the AdS/CFT correspondence", "Correspondence between X and Y" (the word opens a note only
/// before ":" or "to"), "a test for the correspondence" ("for" right before the word only) name no one so (rev78f2,
/// rev78f3). English phrases only: the census's
/// notes and address lists of this population name correspondence in English (OXIDIZED_DESIGN_DIVERGENCES #809).
pub(in crate::base_utilities) fn names_correspondence(text: &str) -> bool {
  CORRESPONDENCE_PHRASE.is_match(&text.to_lowercase())
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

/// The words of a contact's label, lowercase, without hyphens (`E-mail` is `email`): first the e-mail words, then the
/// address words, then the words naming a contact. An e-mail label is an e-mail word, alone or before an address word
/// ([`is_email_label`]); the label an e-mail call has of its own starts with any of them (`Corresponding author:`,
/// `Contact:`, `E-mail address:`, `Correspondence to:`, the 85b r4 re-review's q03, q04, q23; [`names_a_contact`]).
const CONTACT_WORDS: &[&str] = &[
  "email",
  "emails",
  "address",
  "addresses",
  "mail",
  "contact",
  "corresponding",
  "correspondence",
  "author",
  "authors",
];

/// `word`, lowercase, without hyphens or whitespace ([`CONTACT_WORDS`]' form).
fn contact_word_form(word: &str) -> String {
  word
    .chars()
    .filter(|c| !c.is_whitespace() && *c != '-')
    .collect::<String>()
    .to_lowercase()
}

/// Is `text` an e-mail label's words ("E-mail", "Emails", "E-mail address"; [`CONTACT_WORDS`])?
pub(in crate::base_utilities) fn is_email_label(text: &str) -> bool {
  let text = contact_word_form(text);
  let (emails, rest) = CONTACT_WORDS.split_at(2);
  let addresses = &rest[..2];
  emails.iter().any(|email| {
    text
      .strip_prefix(email)
      .is_some_and(|after| after.is_empty() || addresses.contains(&after))
  })
}

/// Is `word` a word of a contact's label ([`CONTACT_WORDS`]: `Corresponding`, `Contact`, `E-mail`, …)?
pub(in crate::base_utilities) fn names_a_contact(word: &str) -> bool {
  CONTACT_WORDS.contains(&contact_word_form(word).as_str())
}

/// A line's leading email label — "E-mail:", "Email:", "e-mails:", "E-mail address:", up to its colon, or the same
/// words without one right before a command — and the rest of the line.
pub(in crate::base_utilities) fn split_email_label(line: &Tokens) -> (Option<Tokens>, Tokens) {
  let tokens = line.unlist_ref();
  let mut text = String::new();
  let mut end = None;
  for (i, t) in tokens.iter().enumerate() {
    match t.code {
      Catcode::LETTER | Catcode::OTHER => {
        let ch = t.to_string();
        if ch == ":" {
          end = Some(i + 1);
          break;
        }
        text.push_str(&ch);
      },
      Catcode::SPACE => text.push(' '),
      Catcode::CS => {
        end = Some(i);
        break;
      },
      _ => break,
    }
  }
  let is_label = is_email_label(&text);
  match end {
    Some(end) if is_label => {
      let label: Vec<Token> = tokens[..end]
        .iter()
        .copied()
        .skip_while(|t| *t == T_SPACE!())
        .collect();
      let label_end = label
        .iter()
        .rposition(|t| !is_spacing(t))
        .map_or(0, |at| at + 1);
      (
        Some(Tokens::new(label[..label_end].to_vec())),
        Tokens::new(
          tokens[end..]
            .iter()
            .copied()
            .skip_while(is_spacing)
            .collect(),
        ),
      )
    },
    _ => (None, line.clone()),
  }
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

/// Space between an email label and its addresses, or between addresses: a space, a tie, or a spacing command
/// (`E-mail:~\email{x}`, `E-mail:\ \email{x}`).
pub(in crate::base_utilities) fn is_spacing(t: &Token) -> bool {
  *t == T_SPACE!()
    || *t == T_ACTIVE!('~')
    || (t.code == Catcode::CS
      && matches!(
        t.to_string().as_str(),
        "\\ "
          | "\\,"
          | "\\;"
          | "\\:"
          | "\\quad"
          | "\\qquad"
          | "\\enskip"
          | "\\enspace"
          | "\\thinspace"
          | "\\space"
      ))
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

/// The commands whose arguments print no letter, each with those arguments as an xparse-style signature — `s` a star,
/// `o` an optional `[…]` argument, `m` a mandatory one — and whether the command prints a space: what the command
/// prints, if anything, is the argument after them. One table for every reader of an author block's printed text
/// ([`email_addresses`], [`address_text`], [`printed_words`], [`link_line_link`], [`mailto_line`]'s label): a colour, a
/// font's name or size, a scale, an angle, a raise, a graphic's file, a skip's length are no letters of an address, a
/// label or a link's text (`\href{mailto:…}{\color{black}…}`, 2604.19440; `\texttt{\fontfamily{pcr}\selectfont
/// xw109@illinois.edu}`, 2609.25669, 2609.35216; `\scalebox{0.9}{…}`, `\fontsize{9}{11}`, the r6 re-review's w14,
/// w15). The signatures are those of xcolor.sty (`\fcolorbox[model]{frame}[model]{background}{text}`, `\colorlet`,
/// `\definecolor`), latex.ltx's font selection (`\fontsize{size}{skip}`, `\usefont{enc}{family}{series}{shape}`) and
/// boxes (`\raisebox{lift}[height][depth]{text}`, `\hspace*{len}`), and graphicx.sty (`\scalebox{h}[v]{text}`,
/// `\rotatebox[keys]{angle}{text}`, `\resizebox*{width}{height}{text}`, `\includegraphics*[…][…]{file}`, all of whose
/// arguments print no letter).
pub(in crate::base_utilities) const NON_PRINTING_ARGUMENTS: &[(&str, &str, bool)] = &[
  ("\\color", "om", false),
  ("\\textcolor", "om", false),
  ("\\colorbox", "om", false),
  ("\\fcolorbox", "omom", false),
  ("\\pagecolor", "om", false),
  ("\\colorlet", "omom", false),
  ("\\definecolor", "ommm", false),
  ("\\fontfamily", "m", false),
  ("\\fontseries", "m", false),
  ("\\fontshape", "m", false),
  ("\\fontencoding", "m", false),
  ("\\fontsize", "mm", false),
  ("\\usefont", "mmmm", false),
  ("\\scalebox", "mo", false),
  ("\\rotatebox", "om", false),
  ("\\raisebox", "moo", false),
  ("\\resizebox", "smm", false),
  ("\\includegraphics", "soom", false),
  ("\\hspace", "sm", true),
  ("\\vspace", "sm", true),
  // (latex.ltx's `\rule[raise]{width}{height}` and `\phantom{…}` print a box, no letter: the r7 re-review's finding 11,
  // probes p1, p2, p4)
  ("\\rule", "omm", false),
  ("\\phantom", "m", true),
  ("\\hphantom", "m", true),
  ("\\vphantom", "m", false),
  // (TeX's `\kern`, `\hskip`, `\vskip` take a dimension or glue, `d`/`g`, no group: probes p5, p6)
  ("\\kern", "d", true),
  ("\\hskip", "g", true),
  ("\\vskip", "g", false),
  // (an icon prints its glyph, no letter: fontawesome5's `\faIcon[style]{name}`, fontawesome's `\faicon[…]{name}` and
  // `\faIconFromMacro{name}`, which their `\faGithub`-style commands expand to, [`icon_macro_options`]; 2601.03661,
  // 2601.06979, the r7 re-review's finding 5)
  ("\\faIcon", "om", false),
  ("\\faicon", "om", false),
  ("\\faIconFromMacro", "m", false),
];

/// Where the command at `at` and its arguments that print no letter end ([`NON_PRINTING_ARGUMENTS`]), the spaces before
/// each argument skipped as TeX skips them, and whether it prints a space; `None` for any other token.
pub(in crate::base_utilities) fn non_printing_end(v: &[Token], at: usize) -> Option<(usize, bool)> {
  non_printing_end_within(v, at, 0)
}

/// [`non_printing_end`], `depth` macros deep in the expansion of a command that prints no word
/// ([`icon_macro_options`]).
fn non_printing_end_within(v: &[Token], at: usize, depth: usize) -> Option<(usize, bool)> {
  let t = v.get(at)?;
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  let found = t.with_str(|name| {
    NON_PRINTING_ARGUMENTS
      .iter()
      .find(|(command, ..)| *command == name)
      .map(|(_, signature, space)| (*signature, *space))
  });
  // (a macro of optional arguments alone whose expansion prints no word, an icon: `\faGithub`, a user's
  // `\newcommand{\yticon}{\textcolor{youtubeRed}{\faYoutube}}`)
  let owned;
  let (signature, space) = match found {
    Some(found) => found,
    None => {
      owned = "o".repeat(icon_macro_options(t, depth)?);
      (owned.as_str(), false)
    },
  };
  let mut i = at + 1;
  for kind in signature.chars() {
    let next = i
      + v[i.min(v.len())..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count();
    match kind {
      's' if v.get(next) == Some(&T_OTHER!("*")) => i = next + 1,
      'o' if v.get(next) == Some(&T_OTHER!("[")) => {
        i = next
          + v[next..]
            .iter()
            .position(|t| *t == T_OTHER!("]"))
            .map_or(v.len() - next, |close| close + 1);
      },
      'm' if next < v.len() => i = skip_group(v, next).max(next + 1),
      'd' | 'g' => i = next + dimension_length(&v[next.min(v.len())..], kind == 'g'),
      _ => {},
    }
  }
  Some((i.min(v.len()), space))
}

/// How many tokens of `v` a dimension takes (`2em`, `-1.5pt`, `\fill`, `0.5\textwidth`), the spaces in it, and with
/// `glue` its `plus`/`minus` parts (`2em plus 1fil`) — TeX's ⟨dimen⟩ and ⟨glue⟩ (tex.web §448, §461), read as written
/// in an author line.
fn dimension_length(v: &[Token], glue: bool) -> usize {
  let text_of = |t: &Token| -> Option<char> {
    matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)
      .then(|| t.with_str(|s| s.chars().next()))
      .flatten()
  };
  // (a sign, digits and a point, then a unit of two letters or a register)
  let dimen = |from: usize| -> usize {
    let mut i = from;
    while v
      .get(i)
      .is_some_and(|t| *t == T_SPACE!() || matches!(text_of(t), Some('+' | '-')))
    {
      i += 1;
    }
    let digits = i;
    while v
      .get(i)
      .is_some_and(|t| matches!(text_of(t), Some(c) if c.is_ascii_digit() || c == '.' || c == ','))
    {
      i += 1;
    }
    while v.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    match v.get(i) {
      Some(t) if t.get_catcode() == Catcode::CS => i + 1,
      Some(_) if i > digits => {
        // (a unit of two letters, `true` before it; `fil`, `fill`, `filll` in glue)
        let letters: String = v[i..]
          .iter()
          .take_while(|t| t.get_catcode() == Catcode::LETTER)
          .take(6)
          .filter_map(text_of)
          .collect();
        let unit = if let Some(more) = letters.strip_prefix("fil") {
          3 + more.chars().take_while(|c| *c == 'l').count().min(2)
        } else if letters.starts_with("true") && letters.len() >= 6 {
          6
        } else if letters.len() >= 2 {
          2
        } else {
          return from;
        };
        // (and the one optional space after a unit, tex.web §443)
        i + unit + usize::from(v.get(i + unit) == Some(&T_SPACE!()))
      },
      _ => from,
    }
  };
  let mut end = dimen(0);
  if end == 0 {
    return 0;
  }
  if glue {
    for keyword in ["plus", "minus"] {
      let mut at = end;
      while v.get(at) == Some(&T_SPACE!()) {
        at += 1;
      }
      let word: String = v[at.min(v.len())..]
        .iter()
        .take(keyword.len())
        .filter_map(text_of)
        .collect();
      if word == keyword {
        let after = dimen(at + keyword.len());
        if after > at + keyword.len() {
          end = after;
        }
      }
    }
  }
  end
}

/// How many optional arguments the command `t` takes when it is a macro taking no other whose expansion prints no
/// word, only commands that print none ([`NON_PRINTING_ARGUMENTS`], an icon: fontawesome5's `\faGithub[]` is
/// `\faIcon[#1]{github}`; 2601.03661's `\faGithub \, \href{…}{…}`, 2601.06979's `\yticon~\href{…}{…}`, the r7
/// re-review's finding 5) and the groups, spaces and spacing around them; `None` for any other command, a macro that
/// expands to nothing among them.
fn icon_macro_options(t: &Token, depth: usize) -> Option<usize> {
  if depth > 3 {
    return None;
  }
  let definition = lookup_definition(t).ok()??;
  let Some(ExpansionBody::Tokens(body)) = definition.get_expansion() else {
    return None;
  };
  let parameters = definition
    .get_parameters()
    .map(|parameters| parameters.get_parameters())
    .unwrap_or_default();
  if parameters
    .iter()
    .any(|parameter| !parameter.optional && !parameter.novalue)
  {
    return None;
  }
  let v = body.unlist_ref();
  let mut icon = false;
  let mut i = 0;
  while i < v.len() {
    let u = &v[i];
    match u.get_catcode() {
      Catcode::BEGIN
      | Catcode::END
      | Catcode::SPACE
      | Catcode::PARAM
      | Catcode::ARG
      | Catcode::MARKER => i += 1,
      _ if is_spacing_command(u) || is_spacing(u) => i += 1,
      Catcode::CS => {
        let (end, _) = non_printing_end_within(v, i, depth + 1)?;
        icon = true;
        i = end.max(i + 1);
      },
      _ => return None,
    }
  }
  icon.then_some(
    parameters
      .iter()
      .filter(|parameter| parameter.optional)
      .count(),
  )
}

/// `v` without the arguments that print no letter ([`NON_PRINTING_ARGUMENTS`]), each command with them gone, or a
/// space where it prints one (a skip). What a command prints, its argument after them, stays.
pub(in crate::base_utilities) fn without_non_printing_arguments(v: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    match non_printing_end(v, i) {
      Some((end, space)) => {
        if space {
          out.push(T_SPACE!());
        }
        i = end.max(i + 1);
      },
      None => {
        out.push(v[i]);
        i += 1;
      },
    }
  }
  out
}

/// `v` with each `\url`'s argument as the command prints it, verbatim: its braces and underscores characters
/// (`\{`, `\}`, `\_`), not a group and a subscript (`\texttt{\url{{co_gy,lyu,funakoshi,oku}@lr.first.iir.isct.ac.jp}}`,
/// a brace list of local parts, 2609.00588).
fn url_arguments_printed(v: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    out.push(v[i]);
    i += 1;
    if v[i - 1] != T_CS!("\\url") {
      continue;
    }
    while v.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    if v.get(i).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
      continue;
    }
    let close = skip_group(v, i).min(v.len());
    out.push(v[i]);
    for t in &v[i + 1..close.saturating_sub(1).max(i + 1)] {
      out.push(match t.get_catcode() {
        Catcode::BEGIN => T_CS!("\\{"),
        Catcode::END => T_CS!("\\}"),
        Catcode::SUB => T_CS!("\\_"),
        _ => *t,
      });
    }
    if close > i + 1 {
      out.push(v[close - 1]);
    }
    i = close;
  }
  out
}

/// Parse an email line into its individual address strings, or `None` if the line
/// is not an email list. Two forms are recognized, and mixed (after skipping
/// wrapper commands and grouping — the visible letter/other/space chars are read,
/// with the printed braces `\{ \}` and underscores `\_`):
/// - **Distributed** — `a@x, b@y, c@z`: every comma item carries its own `@`.
/// - **Grouped brace-expansion** — `\{a,b,c\}@dom`: the local parts in the braces are
///   each expanded against the domain after them → `a@dom, b@dom, c@dom`. This is the
///   compact form co-located authors use for one shared address per person; a line
///   may hold several groups (`{a,b}@x, {c,d}@y`, 2402.02746; `{a,b}@x, c@y`,
///   2410.19160).
///
/// A prose affiliation line ("Dept. of Foo, University of Pisa, Italy") has no `@`,
/// and a place before an address (`Berlin, Germany, foo@bar.de`) is a bare word, so
/// either is rejected and an address is never invented.
pub(in crate::base_utilities) fn email_addresses(line: &Tokens) -> Option<Vec<String>> {
  // (a colour command's colour is no letter of the address: `\href{mailto:…}{\color{black}\texttt{xinhao.zhang},}`,
  // 2604.19440, read `blackxinhao.zhang@…`; a font's name, a scale, a skip's length neither, `\fontfamily{pcr}`,
  // 2609.25669; [`without_non_printing_arguments`])
  // (and a skip's length, printing a space: `xiaobing.gxb@alibaba-inc.com \hspace{25pt}zixuanze@andrew.cmu.edu`,
  // 2107.06049; `\vspace{-1.25ex}`, 2602.09494; a `\url`'s braces print, `\url{{a,b}@x}`, [`url_arguments_printed`])
  let printed = url_arguments_printed(&without_non_printing_arguments(line.unlist_ref()));
  let mut visible = String::new();
  // `\href`'s first argument is its link, not printed (`\href{mailto:a@x}{a@x}`).
  let mut skip_group = false;
  let mut depth = 0usize;
  // (a command's optional arguments are no text: `\makebox[\textwidth][c]{\tt haiyang.wang@huawei.com ~
  // lue.fan@ia.ac.cn}`, 2602.10975)
  let mut optional = false;
  let mut after_command = false;
  for t in &printed {
    if optional {
      optional = *t != T_OTHER!("]");
      continue;
    }
    if after_command && *t == T_OTHER!("[") {
      optional = true;
      continue;
    }
    after_command = t.get_catcode() == Catcode::CS || (after_command && *t == T_SPACE!());
    if skip_group {
      match t.code {
        Catcode::SPACE if depth == 0 => {},
        Catcode::OTHER if depth == 0 && *t == T_OTHER!("*") => {},
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth = depth.saturating_sub(1);
          skip_group = depth > 0;
        },
        _ if depth == 0 => skip_group = false,
        _ => {},
      }
      continue;
    }
    match t.code {
      Catcode::LETTER | Catcode::OTHER => t.with_str(|s| visible.push_str(s)),
      // (a space, and the spacing that parts addresses printed in a row: `\texttt{jl-tong@sjtu.edu.cn}~~~~~
      // \texttt{xyshen@eitech.edu.cn}`, 2603.04592)
      Catcode::SPACE => visible.push(' '),
      Catcode::ACTIVE if *t == T_ACTIVE!('~') => visible.push(' '),
      Catcode::CS if is_spacing_command(t) || is_line_separator(t) => visible.push(' '),
      Catcode::CS if *t == T_CS!("\\href") => skip_group = true,
      // the printed braces of a list of local parts, and an underscore (`wu\_zhiliang`, 2509.10377)
      Catcode::CS => t.with_str(|name| match name {
        "\\{" | "\\lbrace" | "\\textbraceleft" => visible.push('{'),
        "\\}" | "\\rbrace" | "\\textbraceright" => visible.push('}'),
        "\\_" | "\\textunderscore" => visible.push('_'),
        _ => {},
      }),
      _ => {},
    }
  }
  let v = visible.trim();
  if !v.contains('@') {
    return None;
  }
  // Items at commas outside the braces: each a full address, or a braced list of local parts taking the domain after
  // its `}` (`{a,b,c}@dom` → a@dom, b@dom, c@dom). A bare word is no address, so a place before an address
  // (`Berlin, Germany, foo@bar.de`) leaves the line no email list.
  let is_part =
    |p: &str| !p.is_empty() && !p.contains(['@', '{', '}']) && !p.chars().any(char::is_whitespace);
  let mut out = Vec::new();
  let mut rest = v;
  while !rest.trim().is_empty() {
    rest = rest.trim_start();
    let (item, next) = match rest.strip_prefix('{') {
      Some(list) => {
        let close = list.find('}')?;
        let after = &list[close + 1..];
        // (the domain ends at a comma, a semicolon or a space: `{a,b}@x.edu \ \ \ {c,d}@y.edu`, 2603.24730)
        let (domain, next) = match after.find([',', ';', ' ']) {
          Some(end) => (
            &after[..end],
            after[end..].trim_start().trim_start_matches([',', ';']),
          ),
          None => (after, ""),
        };
        // (a sentence's `;` or `.` after it is no part of it: `\{lizixu.cs, …\}@gmail.com;`, 2605.24470)
        let domain = domain.trim().trim_end_matches([';', '.']);
        if !domain.starts_with('@') || !is_part(&domain[1..]) {
          return None;
        }
        // (a `;` parts them as a comma does: `\{hjjang; dychoi; …\}@vclab.kaist.ac.kr`, 2607.16926)
        for local in list[..close]
          .split([',', ';'])
          .map(str::trim)
          .filter(|p| !p.is_empty())
        {
          if !is_part(local) {
            return None;
          }
          out.push(format!("{local}{domain}"));
        }
        (None, next)
      },
      // (an item ends at a comma or a semicolon: `aable@x.edu; \{bbaker, ccole\}@y.edu`, 2604.10316; the r8
      // re-review's §B, probes t1, t2)
      None => {
        let (item, next) = rest.split_once([',', ';']).unwrap_or((rest, ""));
        (Some(item.trim().trim_start_matches("mailto:")), next)
      },
    };
    // (an item is taken whole, as the line prints it: 2410.07147's `vn72@cornell.edu\nsone sj597@…` has no space
    // between its addresses; addresses a space parts are items each, `a@x~~~b@y`, every one of them an address)
    if let Some(item) = item.filter(|item| !item.is_empty()) {
      for item in item.split_whitespace() {
        let item = item.trim_end_matches([';', '.']);
        // (a local part before the `@`: a handle, `@aable`, is no address, the r6 re-review's w4)
        if !item.contains('@') || item.starts_with('@') || item.contains(['{', '}']) {
          return None;
        }
        out.push(item.to_string());
      }
    }
    rest = next;
  }
  (!out.is_empty()).then_some(out)
}
