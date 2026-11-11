//! The text rules on printed ink and the token primitives of `author_block/email_block_calls.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

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

/// Is `text` an e-mail label's words ("E-mail", "Emails", "E-mail address"; [`CONTACT_WORDS`]; "Electronic address")?
pub(in crate::base_utilities) fn is_email_label(text: &str) -> bool {
  let text = contact_word_form(text);
  // (REVTeX's own label, "Electronic address" (revtex4-2.cls `\email`'s note), and "Electronic mail")
  if let Some(after) = text.strip_prefix("electronic")
    && ["address", "addresses", "mail"].contains(&after)
  {
    return true;
  }
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

/// A line's leading e-mail label ([`split_email_label`]) where it names the addresses behind it — the rest a list of
/// addresses, and the whole line either no list or the same addresses read through its label (the glued label,
/// `Email:zz@x.org`, the list `[zz@x.org]` as [`email_addresses`] reads it: the label is still the contacts' name,
/// not part of an address — hep-ph9606235, astro-ph0001190, hep-ph0411269, 2206.00543) — and that rest
/// (`email: a@x, b@y`; `email_line_calls`).
pub(in crate::base_utilities) fn email_line_label(line: &Tokens) -> Option<(Tokens, Tokens)> {
  let (label, rest) = split_email_label(line);
  label
    .filter(|_| {
      email_addresses(&rest)
        .is_some_and(|read| email_addresses(line).is_none_or(|whole| whole == read))
    })
    .map(|label| (label, rest))
}

/// The name of the contacts of an entry's line led by a label of its own — an e-mail label ([`email_line_label`]) or
/// the text before a mailbox link ([`mailbox_link_line`]) — where the class names the entry too (`entry_name`, its
/// `name=…`): the class's name before the line's label, as both print (aa's `\mail{e-mail: x}`, "Correspondence to
/// e-mail:", 0805.1001; `\mail{Contact: \href{mailto:…}{here}}`, "Correspondence to Contact:", review revZ18o N1;
/// REVTeX's `\altaffiliation[Authors for Correspondence: ]{Email: …}`, 2208.10784); the line's label alone where the
/// class names nothing.
pub(in crate::base_utilities) fn entry_line_label(
  entry_name: Option<&[Token]>,
  label: Tokens,
) -> Tokens {
  let Some(name) = entry_name else {
    return label;
  };
  let mut both: Vec<Token> = name
    .iter()
    .copied()
    .skip_while(|t| *t != T_OTHER!("="))
    .skip(1)
    .filter(|t| !matches!(t.get_catcode(), Catcode::BEGIN | Catcode::END))
    .collect();
  if both.last().is_some_and(|t| !is_spacing(t)) {
    both.push(T_SPACE!());
  }
  both.extend(label.unlist());
  Tokens::new(both)
}

/// Space between an email label and its addresses, or between addresses: a space, a tie, or a spacing command
/// (`E-mail:~\email{x}`, `E-mail:\ \email{x}`) — the control return a line ending in `\` leaves among them, which
/// latex.ltx:558 `\def\^^M{\ }` prints as a space (`$^1$Univ A \ \ \ \⏎ $^2$Univ B`, 2510.25684).
pub(in crate::base_utilities) fn is_spacing(t: &Token) -> bool {
  *t == T_SPACE!()
    || *t == T_ACTIVE!('~')
    || (t.code == Catcode::CS
      && matches!(
        t.to_string().as_str(),
        "\\ "
          | "\\\r"
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

/// The commands whose arguments print no letter, each with those arguments as an xparse-style signature — `s` a star,
/// `o` an optional `[…]` argument, `m` a mandatory one — and whether the command prints a space: what the command
/// prints, if anything, is the argument after them. One table for every reader of an author block's printed text
/// ([`email_addresses`], [`address_text`], [`printed_words`], [`link_line_link`]; once `author_block`'s mailbox link line's label, deleted with aas's e-mail line readers, Z18l): a colour, a
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

/// The characters that part the local parts of a braced group sharing one domain (`\{ann, bob\}@x.org`, `\{ann;
/// bob\}@x.org`, `\{ann|bob\}@x.org`): the one separator set of both readers of a group, [`email_addresses`] and the
/// frontmatter observer's (`addresses_in`, `address_runs`).
pub(in crate::base_utilities) const ADDRESS_GROUP_SEPARATORS: [char; 3] = [',', ';', '|'];

/// The characters that part local parts sharing one domain with no braces round them (`a|b@x.org`, `a/b@x.org`): both
/// readers' ([`email_addresses`], the frontmatter observer's `plain_addresses`).
pub(in crate::base_utilities) const UNBRACED_LIST_SEPARATORS: [char; 2] = ['|', '/'];

/// A character of an address's local part as the readers take it (`ann.able`, `j_doe`, `b+x`).
pub(in crate::base_utilities) fn is_local_char(c: char) -> bool {
  c.is_ascii_alphanumeric() || "._+-".contains(c)
}

/// Whether `word` may be a local part of a list written without spaces before one domain (`dzyang,ashishbshah@fb.com`,
/// 2112.00969; `saro,borgani,tornator@oats.inaf.it`, 0810.2791 — the main session's ruling): lowercase letters, digits
/// and `._+-`, a letter among them, no `.` first. A digit run is none (`Tel:5551234,info@x.org` is no `5551234@x.org`,
/// brief 95c r2 B3), nor a capitalised word (`Germany,foo@bar.de`), nor a word naming a contact (`email,ann@x.org`,
/// `contact,bob@a.edu` are no `email@x.org`, `contact@a.edu`: [`names_a_contact`], review rev103a B1), nor one letter, an
/// initial (`d,ashishbshah@fb.com`; rev103a S1, [`ends_a_listed_run`]). The one test of every reader of such a list:
/// [`write_out_unspaced_lists`], the frontmatter observer's `address_runs`.
pub(in crate::base_utilities) fn is_listed_local_part(word: &str) -> bool {
  word.chars().nth(1).is_some()
    && !word.starts_with('.')
    && word.chars().all(is_local_char)
    && !word.chars().any(|c| c.is_ascii_uppercase())
    && word.chars().any(|c| c.is_ascii_alphabetic())
    && !names_a_contact(word)
}

/// Whether the local part `local` of the address after a list of local parts ([`is_listed_local_part`]) may end the
/// list: more than one letter — `smith,j@x.org` prints a surname and an initial, no `smith@x.org` (review rev103a S1).
/// The one test of every reader of such a list, as [`is_listed_local_part`].
pub(in crate::base_utilities) fn ends_a_listed_run(local: &str) -> bool {
  local.chars().nth(1).is_some() && local.chars().all(is_local_char)
}

/// `text` with each list of local parts written without spaces before one domain written out, each part at the domain
/// (`ann@a.edu; dzyang,ashishbshah@fb.com` → `ann@a.edu; dzyang@fb.com,ashishbshah@fb.com`, 2112.00969), as a braced
/// list's are: the one reading of the address readers, the author block's ([`email_addresses`]) and the frontmatter
/// observer's (`plain_addresses`; its `address_runs` reads the list alike), so neither drops a local part the other
/// reads. A list is a run of comma pieces with no space or `;` between, each a local part ([`is_listed_local_part`]) —
/// the first one may follow a label or a bracket glued to it, `Email:ann,bob@x.org` — ending in one address
/// ([`ends_a_listed_run`]); a piece of a braced group is left to the group readers, and a word no address follows is
/// left as it is. A run glued to a word before it that is no local part — `M.R.Pestalozzi,a.chrysostomou,
/// j.l.collett@herts.ac.uk`, astro-ph0612088 — is no list but one word the readers cannot part: left whole, as on base
/// (written out, its first part was lost and the whole word copied onto every carrier of its entry); one whose every
/// part spells an author is written out by [`spelled_unspaced_list`].
pub(in crate::base_utilities) fn write_out_unspaced_lists(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  for chunk in text.split_inclusive(|c: char| c.is_whitespace() || c == ';') {
    let body = chunk.trim_end_matches(|c: char| c.is_whitespace() || c == ';');
    let separator = &chunk[body.len()..];
    if !body.contains(',') || !body.contains('@') || body.contains(['{', '}']) {
      out.push_str(chunk);
      continue;
    }
    let mut pieces: Vec<String> = Vec::new();
    // (the local parts read so far, the first with the label or bracket glued before it)
    let mut pending: Vec<(&str, &str)> = Vec::new();
    // (whether the piece before is a word no local part — a contact's word aside, `email,ann,bob@x.org` — and whether
    // the pending run opened after one)
    let mut after_word = false;
    let mut glued = false;
    let flush = |pending: &mut Vec<(&str, &str)>, pieces: &mut Vec<String>| {
      pieces.extend(pending.drain(..).map(|(before, word)| s!("{before}{word}")));
    };
    for piece in body.split(',') {
      if let Some((local, domain)) = piece.split_once('@') {
        if !pending.is_empty() && !glued && ends_a_listed_run(local) {
          pieces.extend(
            pending
              .drain(..)
              .map(|(before, word)| s!("{before}{word}@{domain}")),
          );
        } else {
          flush(&mut pending, &mut pieces);
        }
        pieces.push(piece.to_string());
        after_word = false;
        continue;
      }
      let (before, word) = match piece.rfind([':', '(', '[', '<']) {
        Some(at) if pending.is_empty() => piece.split_at(at + 1),
        _ => ("", piece),
      };
      if is_listed_local_part(word) {
        if pending.is_empty() {
          glued = after_word && before.is_empty();
        }
        pending.push((before, word));
      } else {
        flush(&mut pending, &mut pieces);
        pieces.push(piece.to_string());
        after_word = !piece.is_empty() && !names_a_contact(piece);
      }
    }
    flush(&mut pending, &mut pieces);
    out.push_str(&pieces.join(","));
    out.push_str(separator);
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
/// - **Unspaced list** — `ann,bob,cy@dom` (no space after the commas): each local part at the
///   domain, as a braced list's ([`write_out_unspaced_lists`]; `ann@a.edu;
///   dzyang,ashishbshah@fb.com`, 2112.00969).
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
        // (a bar between local parts in math, `\{malcolm.crowe $\mid$ carolyn.begg\}@uws.ac.uk`, 2110.02026: a `|`,
        // [`ADDRESS_GROUP_SEPARATORS`])
        "\\mid" | "\\vert" => visible.push('|'),
        _ => {},
      }),
      _ => {},
    }
  }
  let v = visible.trim();
  if !v.contains('@') {
    return None;
  }
  // (a list of local parts written without spaces before one domain, each at it: `ann@a.edu; dzyang,ashishbshah@fb.com`,
  // 2112.00969 — the frontmatter observer's reading too, [`write_out_unspaced_lists`])
  let written = write_out_unspaced_lists(v);
  let v = written.as_str();
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
        // (a `;` parts them as a comma does: `\{hjjang; dychoi; …\}@vclab.kaist.ac.kr`, 2607.16926; so does a `|`,
        // llncs's `\email{\{ivan|carolin\}@…}`, 0902.3918, 1909.00276 — the separators the frontmatter observer's
        // `addresses_in` and `address_runs` part a group at too, [`ADDRESS_GROUP_SEPARATORS`])
        for local in list[..close]
          .split(ADDRESS_GROUP_SEPARATORS)
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
        // (an e-mail label glued to the address, `Email:aable@a.edu`, `(E-mail:bogdan@…` — astro-ph0512555,
        // astro-ph0404330: the address after it; any other colon before the `@` is no address, below)
        let item = match item
          .split_once('@')
          .and_then(|(local, _)| local.rsplit_once(':'))
        {
          Some((label, _)) if is_email_label(label.trim_start_matches('(')) => {
            &item[label.len() + 1..]
          },
          _ => item,
        };
        // (local parts a `|` or a `/` parts before one domain, with no braces: llncs's `\email{robert.holland15|
        // e.chotzoglou16|b.kainz@imperial.ac.uk}`, 1909.00276; `deleo/walrod/vaz@ime.unicamp.br`, hep-ph9906243 — each
        // at the domain, as a braced group's are, [`UNBRACED_LIST_SEPARATORS`])
        if let Some((local, domain)) = item.split_once('@')
          && local.contains(UNBRACED_LIST_SEPARATORS)
          && !domain.contains('@')
        {
          let parts: Vec<&str> = local.split(UNBRACED_LIST_SEPARATORS).collect();
          if parts.iter().all(|part| {
            !part.is_empty()
              && part
                .chars()
                .all(|c| c.is_alphanumeric() || "._+-".contains(c))
          }) {
            out.extend(parts.iter().map(|part| format!("{part}@{domain}")));
            continue;
          }
        }
        // (a local part before the `@`: a handle, `@aable`, is no address, the r6 re-review's w4; nor one holding a
        // colon, a label glued to it — `Email:aable@a.edu`, whose label `split_email_label` reads, emulateapj's
        // astro-ph0512555 shape — OXIDIZED_DESIGN_DIVERGENCES #956)
        if !item.contains('@')
          || item.starts_with('@')
          || item.contains(['{', '}'])
          || item
            .split('@')
            .next()
            .is_some_and(|local| local.contains(':'))
        {
          return None;
        }
        out.push(item.to_string());
      }
    }
    rest = next;
  }
  (!out.is_empty()).then_some(out)
}
