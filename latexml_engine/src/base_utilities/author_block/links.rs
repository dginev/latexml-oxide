//! Link lines in an author block: URLs, link words, mailbox links.

use super::*;

/// `tokens` with each `\quad`/`\qquad` that parts an icon from the link after it a space: a piece of an author block
/// that is only an icon — what prints no word, an icon macro or a graphic ([`without_non_printing_arguments`]) — before
/// a link line ([`is_link_line`]) is that link's, one link line with it, a URL contact ([`is_url_line`]), not an
/// affiliation holding the logo alone (`\github \quad \url{…}`, 2601.17879's
/// `\def\github{\raisebox{-1.5pt}{\includegraphics…}}`; the r8 re-review's finding 9).
pub(in crate::base_utilities) fn join_icons_to_links(tokens: Tokens) -> Tokens {
  let mut v = tokens.unlist();
  let separator = |t: &Token| {
    [
      T_CS!("\\\\"),
      T_CS!("\\newline"),
      T_CS!("\\and"),
      T_CS!("\\And"),
      T_CS!("\\AND"),
      T_CS!("\\quad"),
      T_CS!("\\qquad"),
    ]
    .contains(t)
  };
  let mut delimiters = Vec::new();
  let mut depth = 0usize;
  for (at, t) in v.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && separator(t) => delimiters.push(at),
      _ => {},
    }
  }
  let icon_only = |piece: &[Token]| {
    let piece = Tokens::new(piece.to_vec());
    !prints_nothing(&piece)
      && prints_nothing(&Tokens::new(without_non_printing_arguments(
        piece.unlist_ref(),
      )))
  };
  for (index, at) in delimiters.iter().enumerate() {
    let before = if index == 0 {
      0
    } else {
      delimiters[index - 1] + 1
    };
    let after = delimiters.get(index + 1).copied().unwrap_or(v.len());
    if (v[*at] == T_CS!("\\quad") || v[*at] == T_CS!("\\qquad"))
      && icon_only(&v[before..*at])
      && is_link_line(&Tokens::new(v[at + 1..after].to_vec()))
    {
      v[*at] = T_SPACE!();
    }
  }
  Tokens::new(v)
}

/// The links of a line of several, each a link line of its own ([`is_url_line`]) with the icon or label before it, the
/// spacing and the separators (`|`, `·`, `,`) between them dropped (`\faGithub~\href{…}{Code} | \yticon~\href{…}{Demo
/// Video}`, 2601.06979; the r7 re-review's finding 5): a URL contact each. The line whole when it holds one link, or a
/// piece of it is no such link line (a row of linked names).
pub(in crate::base_utilities) fn link_run_lines(line: Tokens) -> Vec<Tokens> {
  let v = line.unlist_ref();
  let mut ends = Vec::new();
  let mut depth = 0usize;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if depth == 0 && (t == T_CS!("\\href") || t == T_CS!("\\url")) {
      let (_, (_, after_second)) = link_arguments(v, i);
      let (_, close) = link_printed_argument(v, i);
      i = if t == T_CS!("\\href") {
        after_second
      } else {
        close
      }
      .max(i + 1);
      ends.push(i);
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    i += 1;
  }
  if ends.len() < 2 {
    return vec![line];
  }
  let separator = |t: &Token| {
    t.get_catcode() == Catcode::MATH
      || is_spacing(t)
      || is_spacing_command(t)
      || [",", ";", "|", "·", "/", "•"]
        .iter()
        .any(|mark| *t == T_OTHER!(*mark))
      || *t == T_CS!("\\textbar")
      || *t == T_CS!("\\textbullet")
      || *t == T_CS!("\\cdot")
  };
  let mut pieces = Vec::with_capacity(ends.len());
  let mut start = 0;
  for (at, end) in ends.iter().enumerate() {
    let end = if at + 1 == ends.len() { v.len() } else { *end };
    let mut from = start;
    while from < end && separator(&v[from]) {
      from += 1;
    }
    pieces.push(Tokens::new(v[from..end].to_vec()));
    start = end;
  }
  if pieces.iter().all(is_url_line) {
    pieces
  } else {
    vec![line]
  }
}

/// The words that name what a link leads to — a page, a project, its code or data — and no institution or person
/// (`Project Page`, `GitHub Repository`, `Hugging Face Demo`, `Substack Article`: 2602.22227, 2604.24697, 2601.18810,
/// 2604.15404, the r6 re-review's finding 2; `Paper Website`, the r8 re-review's finding 2, probe u3), as [`link_words`]
/// reads them.
const LINK_WORDS: &[&str] = &[
  "page",
  "webpage",
  "homepage",
  "website",
  "site",
  "url",
  "link",
  "code",
  "repository",
  "repo",
  "github",
  "demo",
  "dataset",
  "data",
  "article",
  "blog",
  "video",
  "model",
  "project",
  "paper",
];

/// Whether `text` holds a word of [`LINK_WORDS`], in any case, singular or plural.
fn link_words(text: &str) -> bool {
  text
    .split(|c: char| !c.is_alphanumeric())
    .map(str::to_lowercase)
    .any(|word| {
      let word = word
        .strip_suffix('s')
        .filter(|_| word.len() > 3)
        .unwrap_or(&word);
      LINK_WORDS.contains(&word)
    })
}

/// Whether the web address `target` reaches a page below its site's root: a path after its host (`https://x.org/proj`,
/// `github.com/a/b`), not the bare domain an institution's link is (`https://www.google.com`, `https://x.ai/`).
fn target_has_path(target: &str) -> bool {
  let rest = target.split_once("://").map_or(target, |(_, rest)| rest);
  rest
    .split_once('/')
    .is_some_and(|(_, path)| !path.trim_matches(['/', '#', '?']).is_empty())
}

/// Whether `line` is a link line ([`is_link_line`]) to a web page, a URL contact of the author block
/// ([`url_contact_call`]), by what the link says and where it goes, not by the shape of its words: not to a mailbox
/// (`\textbf{Correspondence:} \href{mailto:…}{…}`, 2601.00240, 2605.18563, an address and no URL: an e-mail line,
/// [`mailto_line`]); a `\url`, or an `\href` printing a web address or a link word before or in its text
/// ([`link_words`]: `\href{…}{Project Page}`, `Demo Code:~\href{…}{here}`, 2602.09509); not one printing an institution,
/// that institution linked, an affiliation (`\href{https://launchsafe.com}{\textbf{LaunchSafe Research}}`, 2606.26479,
/// the r5 re-review's u32, u33, u35), nor a person's name, that name (`\textit{Author:} \href{…}{Masashi Sekine}`,
/// 2603.22058); else a link printing no word (an icon) or to a page below a site's root is one, and a link to a bare
/// domain is the institution it prints (`\href{https://www.google.com}{Google}`, `\href{https://x.ai}{xAI}`, the r6 re-review's x10, w8).
pub(in crate::base_utilities) fn is_url_line(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  let (target, printed) = link_texts(v, at);
  if links_a_mailbox(&target, &printed) {
    return false;
  }
  if v[at] == T_CS!("\\url") || reads_as_web_address(&printed) {
    return true;
  }
  let label = visible_name_text(&printed_words(&v[..at]));
  let (open, close) = link_printed_argument(v, at);
  let printed_tokens = Tokens::new(printed_words(&v[open..close]));
  // (an institution holding a link word is that institution linked: `\href{…}{Data Science Institute}`, the r7
  // re-review's finding 4, probe n6)
  if reads_as_affiliation(&printed_tokens) {
    return false;
  }
  if link_words(&printed) || link_words(&label) {
    return true;
  }
  if name_count(&visible_name_text(printed_tokens.unlist_ref())) > 0 {
    return false;
  }
  // (an icon, a link printing no word, is one: 2604.10930; and one printing a word that names no organization, `here`,
  // the r7 re-review's finding 10)
  !printed.chars().any(char::is_alphanumeric)
    || target_has_path(&target)
    || !names_an_organization(&visible_name_text(printed_tokens.unlist_ref()), &target)
}

/// Whether `line` is a link line printing an institution, no URL line ([`is_url_line`]): its printed text reads as an
/// affiliation (`\href{https://launchsafe.com}{\textbf{LaunchSafe Research}}`, 2606.26479), or it links a bare domain
/// and prints no name (`\href{https://www.google.com}{Google}`, the r6 re-review's x9, x10) — not a linked name
/// (`Advisor: \href{…}{Carl Doe}`, the r6 re-review's w5).
pub(in crate::base_utilities) fn link_prints_an_institution(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  let (target, printed) = link_texts(v, at);
  if is_url_line(line) || links_a_mailbox(&target, &printed) {
    return false;
  }
  let (open, close) = link_printed_argument(v, at);
  let printed = Tokens::new(printed_words(&v[open..close]));
  let text = visible_name_text(printed.unlist_ref());
  reads_as_affiliation(&printed)
    || (name_count(&text) == 0
      && names_an_organization(&text, &target)
      && !target_has_path(&target))
}

/// Whether a bare domain's link text names its organization, as such a name is written — each word with a capital
/// (`Google`, `xAI`) — or as the domain `target` it links spells it in a handle
/// (`\href{https://www.deepseek.com}{deepseek-ai}`, the r8 re-review's finding 9, probe s14; not a homepage's bare
/// `{annable}`, the 77y review's finding 4), not a word of the sentence around it (`\href{https://ann.org}{here}`, a
/// link: the r7 re-review's finding 10, probes n13, p9).
fn names_an_organization(text: &str, target: &str) -> bool {
  let capitalized = !text.trim().is_empty()
    && text
      .split_whitespace()
      .all(|word| word.chars().any(char::is_uppercase));
  // (the domain's name, the label before its top-level one: "deepseek" of `https://www.deepseek.com`)
  let host = target
    .split_once("://")
    .map_or(target, |(_, rest)| rest)
    .split(['/', '?', '#'])
    .next()
    .unwrap_or_default()
    .to_lowercase();
  let labels: Vec<&str> = host.split('.').filter(|label| !label.is_empty()).collect();
  let spelled: String = text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .collect::<String>()
    .to_lowercase();
  // (an organization's handle, written with a mark no word holds — `deepseek-ai` —, not a person's: a homepage's link
  // printing the name its domain spells, `\href{https://www.annable.com}{annable}`, is a URL contact; the 77y review's
  // finding 4, probes d1, d2)
  // (the label's letters and digits, as the text's are read: `ml-group.org`, `{ml-group}`)
  let label: String = labels
    .len()
    .checked_sub(2)
    .map(|at| labels[at].chars().filter(|c| c.is_alphanumeric()).collect())
    .unwrap_or_default();
  let domain_named = label.chars().count() >= 3
    && spelled.contains(label.as_str())
    && text
      .chars()
      .any(|c| !c.is_alphabetic() && !c.is_whitespace());
  capitalized || domain_named
}

/// Whether `line` is a link printing one word whose letters are exactly a spelling of an author's name (`keys`: a joined
/// spelling or the initials, [`NameKeys`]): the author's own handle, a homepage — `\href{https://annable.com}{ann-able}`,
/// `\href{https://www.annable42.com}{annable42}`, `{AnnAble}` under Ann Able (the rev77y2 re-review's finding 5, probes
/// d5, d9, d12; `deepseek-ai` no name spells stays an affiliation). A name's words running inside an organization's
/// name are no spelling of it (`{Huawei}` under Hua Wu and Xiaoming Wei, `{Samsung}` under Sam Smith, `{able-lab}`
/// under Ann Able: affiliations, the rev77y3 re-review's finding 2, probes h1-h9). An organization whose name is exactly
/// an author's spelling (`{Huawei}` under Hua Wei, `{HW}` under Hua Wu) reads as that author's handle: the link stays on
/// the name it prints under, as a URL contact (rev77y4 probes h20, h21; SYNC residual).
pub(in crate::base_utilities) fn link_prints_a_handle(line: &Tokens, keys: &[NameKeys]) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let (_, printed) = link_texts(&line.unlist_ref()[..end], at);
  let printed = printed.trim();
  let letters: String = printed
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect::<String>()
    .to_lowercase();
  !letters.is_empty()
    && !printed.contains(char::is_whitespace)
    && !printed.contains('@')
    && keys
      .iter()
      .flat_map(NameKeys::spellings)
      .any(|keys| keys.joined.contains(&letters) || keys.initials.contains(&letters))
}

/// Whether `line` is a link line ([`is_url_line`]) that prints a web address, a link word or a label before its link:
/// a `\url`, an `\href` printing a URL, `\href{…}{Project Page}`, `Project page: \href{…}{here}`. A link line opening an
/// author block or an `\and` column is only one so, a linked name there its author
/// (`\href{https://orcid.org/…}{\includegraphics{orcid.pdf}\hspace{1mm}Eric Rossetto}`, 2608.24427, 2602.15968,
/// 2603.04124); a column of a "Project Page" link alone is the block's, no author (the r6 re-review's w22). What the
/// link prints decides, never the line under it: a name whose surname is a link word stays a name
/// ([`link_words_beyond_a_surname`]; the rev77y2 re-review's finding 3).
pub(in crate::base_utilities) fn prints_a_url(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  let (_, printed) = link_texts(v, at);
  // (a printed text that reads as a name is that name linked, unless it shows a scheme or `www.`, and no word of it is a
  // link word but a name's surname: `\href{…}{Lucy Page}`, `\href{…}{Edward A.G. Schuur}`, 2601.06085;
  // the r7 re-review's findings 2 and 4 — not `\href{…}{Project Page}`, `{Model Card}`, `{Paper Website}`, `{Hugging
  // Face Demo}`, the r8 re-review's finding 2, probes u1, u3, u4)
  if name_count(&printed) > 0
    && !printed.contains("://")
    && !printed
      .split_whitespace()
      .any(|word| word.starts_with("www."))
    && !link_words_beyond_a_surname(&printed)
  {
    return false;
  }
  is_url_line(line)
    && (v[at] == T_CS!("\\url")
      || reads_as_web_address(&printed)
      || link_words(&printed)
      || v[..at]
        .iter()
        .any(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)))
}

/// Whether a link line prints a bare web address (`\href{https://sherpa.ai/}{Sherpa.ai}`), no scheme or `www.` shown: the
/// way an organization's name is printed where no one else names the block (2609.25082).
pub(in crate::base_utilities) fn link_prints_a_bare_address(line: &Tokens) -> bool {
  let Some((at, end)) = link_line_link(line) else {
    return false;
  };
  let v = &line.unlist_ref()[..end];
  if v[at] == T_CS!("\\url") {
    return false;
  }
  let (_, printed) = link_texts(v, at);
  reads_as_web_address(&printed)
    && !printed.contains("://")
    && !printed
      .split_whitespace()
      .any(|word| word.starts_with("www."))
}

/// Whether `text` is a bare e-mail address, a mailbox and no web page: a local part, an `@`, a domain with a dot, and
/// no `/` (`binzhu@smu.edu.sg` as the ACL template's `\href{binzhu@smu.edu.sg}{binzhu@smu.edu.sg}` writes it, without
/// `mailto:`; 2604.04847, 2605.29064, 2605.29893, 2606.01600) — not a handle (`@aable`, the r6 re-review's w21), nor a
/// list of them (`{a,b}@x.edu` is no one address, [`email_addresses`] reads it: 2609.00588).
pub(in crate::base_utilities) fn is_bare_address(text: &str) -> bool {
  let Some((local, domain)) = text.trim().split_once('@') else {
    return false;
  };
  !local.is_empty()
    && !local.contains([',', ';', '{', '}', ' '])
    && !text.contains('/')
    && domain.split('.').filter(|part| !part.is_empty()).count() >= 2
    && !domain.contains(['@', ' ', ','])
}

/// Whether a link to `target` printing `printed` reaches a mailbox: to `mailto:`, or a bare address
/// ([`is_bare_address`]) either way, its target no web page (`\href{https://x.com/aable}{@aable}` is a profile's, the r6
/// re-review's w21).
pub(in crate::base_utilities) fn links_a_mailbox(target: &str, printed: &str) -> bool {
  target.starts_with("mailto:")
    || (!target.contains("://") && (is_bare_address(target) || is_bare_address(printed)))
}

/// Whether `text` prints a web address: a word with `://`, opening with `www.`, with a `/` between two letters or digits
/// (`github.io/traj_evo_search`), or with a `.` between two whose last label is a top-level domain of two letters or
/// more (`x.org`), and no `@` — not a run of dotted initials (`Edward A.G. Schuur`, 2601.06085; `T.J.C. Bezerra`,
/// 2606.03745; the r7 re-review's finding 2), nor a version number (`v1.2`).
pub(in crate::base_utilities) fn reads_as_web_address(text: &str) -> bool {
  text.split_whitespace().any(|word| {
    let between = |c: u8| {
      word
        .as_bytes()
        .windows(3)
        .any(|w| w[0].is_ascii_alphanumeric() && w[1] == c && w[2].is_ascii_alphanumeric())
    };
    let initials = word
      .as_bytes()
      .windows(3)
      .any(|w| w[0].is_ascii_uppercase() && w[1] == b'.' && w[2].is_ascii_uppercase());
    let domain = word
      .trim_end_matches(|c: char| !c.is_alphanumeric())
      .rsplit_once('.')
      .is_some_and(|(_, last)| last.len() >= 2 && last.chars().all(|c| c.is_ascii_alphabetic()));
    !word.contains('@')
      && (word.contains("://")
        || word.starts_with("www.")
        || between(b'/')
        || (between(b'.') && domain && !initials))
  })
}

/// Whether `text` holds a link word ([`LINK_WORDS`]) other than a name's surname: `Model Card`, `Paper Website`,
/// `Hugging Face Demo`, `Interactive Online Demo`, `Project Page` (the r8 re-review's finding 2, probes u1, u3, u4; the
/// rev77y2 re-review's finding 3, probes l1-l3), wherever the column puts it. A word before the last is one whole, as
/// spaces part it (`Anne-Page Smith` a name, probe l7); the last is the surname, no link word, where the name reads as
/// one around it — two words (`Lucy Page`, `Mark Link`, `Hans-Peter Link`, `Lucy Page-Smith`: the r7 re-review's
/// finding 4), an initial before it (`Lawrence E. Page`), or more words before a link word that is also a surname
/// ([`SURNAME_LINK_WORDS`]: `Mary Ann Code`, `Lawrence Edward Page`, `Juan Carlos de la Page`, alone in their column or
/// over an e-mail or a place: the 77y review's finding 1, probes a1-a3, a7, a8; the rev77y2 re-review's finding 3,
/// probes l4-l6; no paper in either scan). A two-word text stays a name (`Source Code`, `Online Demo` alone in a column:
/// no corpus witness, probes a5, a6; a SYNC residual), and so does a three-word one ending in such a surname (`Full
/// Source Code`, `Google Scholar Page`, `Hugging Face Link`: the rev77y3 re-review's finding 5, probes s1, s3, s4, s7;
/// no author block linking such a text in the 2025-2026 scan; the same residual). The surname is read as a link word is,
/// a plural's `s` dropped (`Mary Ann Pages`), after a name suffix is set aside ([`NAME_SUFFIXES`]: `Lawrence Page Jr.`;
/// the rev77y3 re-review's findings 4 and 6, probes s9, s8).
fn link_words_beyond_a_surname(text: &str) -> bool {
  let mut words: Vec<&str> = text.split_whitespace().collect();
  // (a name suffix after the surname is no surname: `Lawrence Page Jr.`, the rev77y3 re-review's finding 6, probe s8)
  if words.len() > 2
    && words
      .last()
      .is_some_and(|word| NAME_SUFFIXES.contains(&word.trim_end_matches(',')))
  {
    words.pop();
  }
  let Some((last, before)) = words.split_last() else {
    return false;
  };
  let initial = |word: &&str| {
    word.ends_with('.')
      && word.chars().next().is_some_and(char::is_uppercase)
      && word.chars().all(|c| c.is_alphabetic() || c == '.')
      && word.chars().filter(|c| c.is_alphabetic()).count() <= 2
  };
  // (a word whole, as spaces part it: no part of `Anne-Page`; lowercased, a plural's `s` dropped)
  let stem = |word: &str| {
    let word = word
      .trim_matches(|c: char| !c.is_alphanumeric())
      .to_lowercase();
    match word.strip_suffix('s') {
      Some(stem) if word.len() > 3 => stem.to_string(),
      _ => word,
    }
  };
  let link_word = |word: &str| LINK_WORDS.contains(&stem(word).as_str());
  // (the last word's link words, a part of it each, `Page-Smith`, all surnames, read as a link word is: `Mary Ann
  // Pages`, the rev77y3 re-review's finding 4, probe s9)
  let surname = || {
    last
      .split(|c: char| !c.is_alphanumeric())
      .filter(|part| link_word(part))
      .all(|part| SURNAME_LINK_WORDS.contains(&stem(part).as_str()))
  };
  before.iter().any(|word| link_word(word))
    || (link_words(last) && !(before.len() == 1 || before.iter().any(initial) || surname()))
}

/// The link words ([`LINK_WORDS`]) that are also surnames (`Lawrence Edward Page`, `Mary Ann Code`, `Hans Link`): a
/// name's last word, never a link's ([`link_words_beyond_a_surname`]).
const SURNAME_LINK_WORDS: &[&str] = &["page", "link", "code"];

/// A link line ([`link_line_link`]) to a mailbox (`\textbf{Correspondence:} \href{mailto:x@y}{x@y}`, `Email:
/// \href{mailto:…}{…}`), read as a labelled e-mail line (user ruling 2026-10-20, batch 77p r8: 2601.00240, 2601.07974,
/// 2604.23002, 2606.03014, 2605.18563): the text before the link, as it prints without its font ("Correspondence:"),
/// is the label, and the link, kept a link, the address ([`address_text`] reads the one it prints). `None` for any
/// other line.
pub(in crate::base_utilities) fn mailto_line(
  line: &Tokens,
) -> Option<(Option<Tokens>, EmailAddress)> {
  let (at, end) = link_line_link(line)?;
  let v = line.unlist_ref();
  // (the link's target and printed text: to a mailbox, or a bare address without `mailto:`, the ACL template's
  // `\textbf{Correspondence:} \href{binzhu@smu.edu.sg}{binzhu@smu.edu.sg}`, 2604.04847, 2605.29064, 2605.29893,
  // 2606.01600)
  let (target, printed) = link_texts(&v[..end], at);
  if !links_a_mailbox(&target, &printed) {
    return None;
  }
  let mut label: Vec<Token> = Vec::new();
  for t in &without_non_printing_arguments(&v[..at]) {
    match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => label.push(*t),
      _ if (t.get_catcode() == Catcode::SPACE || is_spacing(t))
        && label.last().is_some_and(|last| *last != T_SPACE!()) =>
      {
        label.push(T_SPACE!())
      },
      _ => {},
    }
  }
  while label.last() == Some(&T_SPACE!()) {
    label.pop();
  }
  Some((
    (!label.is_empty()).then(|| Tokens::new(label)),
    EmailAddress {
      address: Tokens::new(v[at..end].to_vec()),
      call:    None,
    },
  ))
}

/// The e-mail addresses of an author block's line, as text: a line of addresses (`a@x, b@y`, `{a,b}@dom`) or a
/// labelled one (`Email: \email{a@x}`); none for any other line.
pub(in crate::base_utilities) fn line_address_texts(line: &Tokens) -> Result<Vec<String>> {
  if let Some(addresses) = email_addresses(line) {
    return Ok(addresses);
  }
  Ok(
    line_labelled_emails(line)?
      .map(|(_, addresses)| {
        addresses
          .iter()
          .map(|address| address_text(&address.address))
          .collect()
      })
      .unwrap_or_default(),
  )
}

/// Which names (by index into `name_keys`) one of `addresses` spells ([`spelled_owner`]).
pub(in crate::base_utilities) fn names_spelled(
  name_keys: &[NameKeys],
  addresses: &[String],
) -> Vec<bool> {
  let mut spelled = vec![false; name_keys.len()];
  for address in addresses {
    if let Some(owner) = spelled_owner(name_keys, address) {
      spelled[owner] = true;
    }
  }
  spelled
}
