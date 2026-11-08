//! Link lines in an author block: URLs, link words, mailbox links.

use super::*;

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
