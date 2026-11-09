//! The text rules on printed ink and the token primitives of `author_block/links.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08; `is_url_line`, `link_prints_an_institution` and the
//! rules they read, by FB2b1).

use super::*;

/// Whether `line` is only a link, `\href{…}{…}` or `\url{…}`, with the groups, size or font it is set in, the spacing
/// that prints nothing (`\href{https://jnnan.github.io/trumans/}{…} \vspace{-21pt}`, 2403.08629;
/// `{\footnotesize \url{…}}`, 2602.22394; `{\tt{\small{\url{…}}}}`, 2602.23802), and a label ending in a colon before it
/// (`Demo Code:~\url{…}`, 2602.09509).
pub(in crate::base_utilities) fn is_link_line(line: &Tokens) -> bool {
  link_line_link(line).is_some()
}

/// [`link_prints_an_institution`] on a link as it printed, `\href{target}{printed}`: the reading's link contact, its
/// target and text (`resolver/blocks.rs`'s block links, review revFB2a finding 3).
pub(in crate::base_utilities) fn href_prints_an_institution(target: &str, printed: &str) -> bool {
  let mut line = vec![T_CS!("\\href"), T_BEGIN!()];
  line.extend(ExplodeText!(target));
  line.extend([T_END!(), T_BEGIN!()]);
  line.extend(ExplodeText!(printed));
  line.push(T_END!());
  link_prints_an_institution(&Tokens::new(line))
}

/// The words that name what a link leads to — a page, a project, its code or data — and no institution or person
/// (`Project Page`, `GitHub Repository`, `Hugging Face Demo`, `Substack Article`: 2602.22227, 2604.24697, 2601.18810,
/// 2604.15404, the r6 re-review's finding 2; `Paper Website`, the r8 re-review's finding 2, probe u3), as [`link_words`]
/// reads them.
pub(in crate::base_utilities) const LINK_WORDS: &[&str] = &[
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
pub(in crate::base_utilities) fn link_words(text: &str) -> bool {
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
/// (`url_contact_call`), by what the link says and where it goes, not by the shape of its words: not to a mailbox
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

/// Where the one link of a link line ([`is_link_line`]) stands among its tokens, from its command to the end of its
/// arguments; `None` for any other line.
pub(in crate::base_utilities) fn link_line_link(line: &Tokens) -> Option<(usize, usize)> {
  let v = line.unlist_ref();
  let (start, end) = (leading_spacing_end(v), spacing_end(v));
  // (a line of spacing alone is no link: 2510.07961, 2607.21993)
  if start >= end {
    return None;
  }
  let v = &v[start..end];
  let mut links = 0;
  let mut link = (0, 0);
  // (the last visible character of a label before the link)
  let mut label: Option<char> = None;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if matches!(
      t.get_catcode(),
      Catcode::SPACE | Catcode::BEGIN | Catcode::END
    ) || is_spacing_command(&t)
      || is_spacing(&t)
      || (t.get_catcode() == Catcode::CS
        && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s) || s == "\\texttt"))
    {
      i += 1;
    } else if let Some((end, _)) = non_printing_end(v, i) {
      // (a colour, a font, a scale, an icon the link is set in or beside prints no word: `{\color{orange}\textbf{\url{…}}}`,
      // `\textcolor{blue}{\url{…}}`, [`NON_PRINTING_ARGUMENTS`])
      i = end.max(i + 1);
    } else if t == T_CS!("\\href") || t == T_CS!("\\url") {
      if label.is_some_and(|last| last != ':') {
        return None;
      }
      // (a link that prints nothing is no link of the line: 2601.02680's `\href{…}{…} \url{}`)
      let (open, close) = link_printed_argument(v, i);
      if v[open..close].iter().all(|t| {
        matches!(
          t.get_catcode(),
          Catcode::BEGIN | Catcode::END | Catcode::SPACE
        )
      }) {
        i = close.max(i + 1);
        continue;
      }
      links += 1;
      let at = i;
      // (its arguments, the spaces before each skipped as TeX skips them: `\href{…}⏎{…}`, 2605.06223; none after a
      // bare `\href` that ends the line)
      let argument = |at: usize| {
        let at = at
          + v.get(at..).map_or(0, |rest| {
            rest.iter().take_while(|t| **t == T_SPACE!()).count()
          });
        skip_group(v, at)
      };
      i = argument(i + 1);
      if t == T_CS!("\\href") {
        i = argument(i);
      }
      link = (start + at, start + i.min(v.len()));
    } else if links == 0 && matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
      label = t.with_str(|s| s.chars().last());
      i += 1;
    } else {
      return None;
    }
  }
  (links == 1).then_some(link)
}

/// `v` without what prints no word ([`without_non_printing_arguments`]): a colour's arguments, a graphic with its
/// arguments (an ORCID icon, `\includegraphics[scale=0.06]{orcid.pdf}`), a skip's length (`\hspace{1mm}`, a space).
pub(in crate::base_utilities) fn printed_words(v: &[Token]) -> Vec<Token> {
  without_non_printing_arguments(v)
}

/// Where the printed argument of the link at `at` in `v` stands: an `\href`'s second, a `\url`'s one.
pub(in crate::base_utilities) fn link_printed_argument(v: &[Token], at: usize) -> (usize, usize) {
  let (target, printed) = link_arguments(v, at);
  if v.get(at) == Some(&T_CS!("\\href")) {
    printed
  } else {
    target
  }
}

/// Where the first two arguments after the link command at `at` in `v` stand, the spaces before each skipped as TeX
/// skips them; empty past the end.
pub(in crate::base_utilities) fn link_arguments(
  v: &[Token],
  at: usize,
) -> ((usize, usize), (usize, usize)) {
  let end = v.len();
  let argument = |from: usize| -> (usize, usize) {
    let from = from.min(end);
    let open = from + v[from..].iter().take_while(|t| **t == T_SPACE!()).count();
    (open, skip_group(v, open).clamp(open, end))
  };
  let first = argument(at + 1);
  (first, argument(first.1))
}

/// The target and the printed text of the link at `at` in `v` (`\href{target}{printed}`; a `\url`'s argument is both),
/// as their letters, other characters and spaces read, what prints no word aside ([`printed_words`]: a colour, an ORCID
/// icon's `\includegraphics[scale=0.06]{orcid.pdf}`, a skip's length).
pub(in crate::base_utilities) fn link_texts(v: &[Token], at: usize) -> (String, String) {
  let text = |v: &[Token]| -> String {
    let mut text = String::new();
    for t in printed_words(v) {
      match t.get_catcode() {
        Catcode::LETTER | Catcode::OTHER => t.with_str(|s| text.push_str(s)),
        Catcode::SPACE => text.push(' '),
        Catcode::ACTIVE if t == T_ACTIVE!('~') => text.push(' '),
        _ => {},
      }
    }
    text.trim().to_string()
  };
  let ((open, close), _) = link_arguments(v, at);
  let target = text(&v[open..close]);
  let (open, close) = link_printed_argument(v, at);
  (target, text(&v[open..close]))
}

/// Whether `address` is a role mailbox, no person's (RFC 2142 "Mailbox Names for Common Services, Roles and
/// Functions", and `contact@`, `office@`, `admin@`): `info@x.edu` beside `bbaker@x.edu` is not Ann Able's.
pub(in crate::base_utilities) fn is_role_mailbox(address: &str) -> bool {
  let local = address
    .split('@')
    .next()
    .unwrap_or_default()
    .trim()
    .to_lowercase();
  matches!(
    local.as_str(),
    "info"
      | "marketing"
      | "sales"
      | "support"
      | "abuse"
      | "noc"
      | "security"
      | "postmaster"
      | "hostmaster"
      | "usenet"
      | "news"
      | "webmaster"
      | "www"
      | "uucp"
      | "ftp"
      | "contact"
      | "office"
      | "admin"
  )
}

/// Whether the local part of the e-mail address `address` ([`address_text`]) holds a list the address reader could not
/// part (`\{luoyanyan,…,fengjunlan\}it@chinamobile.com`, 2605.27805; the r8 re-review's finding 4, probe t5): its
/// letters run together spell one of the names it lists, so it spells no one, the block's (user ruling R10). One local
/// part in printed braces is no list: `\{raffaele.perego\}@isti.cnr.it` spells Raffaele Perego (2403.13468).
pub(in crate::base_utilities) fn lists_local_parts(address: &str) -> bool {
  address.split_once('@').is_some_and(|(local, _)| {
    let lone = local
      .strip_prefix('{')
      .and_then(|inner| inner.strip_suffix('}'))
      .is_some_and(|inner| !inner.is_empty() && !inner.contains(['{', '}']));
    local.contains([',', ';', '|']) || (local.contains(['{', '}']) && !lone)
  })
}
