//! The text rules on printed ink and the token primitives of `author_block/links.rs`, moved unchanged (MV1,
//! user ruling 2026-11-08 "knowledge in, driver out").

use super::*;

/// Whether `line` is only a link, `\href{…}{…}` or `\url{…}`, with the groups, size or font it is set in, the spacing
/// that prints nothing (`\href{https://jnnan.github.io/trumans/}{…} \vspace{-21pt}`, 2403.08629;
/// `{\footnotesize \url{…}}`, 2602.22394; `{\tt{\small{\url{…}}}}`, 2602.23802), and a label ending in a colon before it
/// (`Demo Code:~\url{…}`, 2602.09509).
pub(in crate::base_utilities) fn is_link_line(line: &Tokens) -> bool {
  link_line_link(line).is_some()
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
