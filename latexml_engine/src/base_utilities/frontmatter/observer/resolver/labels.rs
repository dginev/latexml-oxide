//! The resolver's text readers (OXIDIZED_DESIGN_DIVERGENCES #837): attribute values, mark labels and their owners
//! (Perl's prefix chain, fuzzy labels by surname, glued labels parted), names folded and spelled, legends, addresses
//! and contact roles.

use super::*;

/// `key=value` of an attribute string (`role=affiliation,label=a`), braces stripped: a repeated key's last value, as
/// `KeyVals::get_value` reads it (the ledger keeps every pair in source order: `role=affiliation,role=address`).
pub(super) fn attr_value<'a>(attrs: &'a str, key: &str) -> Option<&'a str> {
  split_top_commas(attrs).into_iter().rev().find_map(|kv| {
    let (k, v) = kv.split_once('=')?;
    (k.trim() == key).then(|| {
      v.trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim()
    })
  })
}

/// `text` split at its commas outside braces.
pub(super) fn split_top_commas(text: &str) -> Vec<&str> {
  let mut parts = Vec::new();
  let (mut depth, mut start) = (0i32, 0usize);
  for (i, c) in text.char_indices() {
    match c {
      '{' => depth += 1,
      '}' => depth -= 1,
      ',' if depth == 0 => {
        parts.push(&text[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  parts.push(&text[start..]);
  parts
}

/// LaTeX's `\@fnsymbol` characters, the math asterisk as `*`.
pub(super) fn is_symbol_char(c: char) -> bool {
  matches!(c, '*' | '\u{2217}' | '†' | '‡' | '§' | '¶' | '‖')
}

/// One mark's labels (ruling 1's split): at commas and spaces; a run of one footnote symbol is one label (`**`), a
/// symbol glued to a label another (`*1` → `*`, `1`); parentheses and brackets around a label dropped (`(a)`); the
/// math asterisk `∗` read as `*`.
pub fn mark_labels(label: &str) -> Vec<String> {
  // (a `\ref` mark's key, and a `fuzzy:<name>` label — svjour's `\institute` entry after `\at`, a name with its
  // spaces — are one label each, never split: review revfms2a2 finding 2, 0807.3898, 2403.09642)
  if label.starts_with("ref:") || label.starts_with("fuzzy:") {
    return vec![label.trim().to_string()];
  }
  let mut labels = Vec::new();
  for part in label.split(|c: char| c == ',' || c == ';' || c.is_whitespace()) {
    let part = part.trim_matches(|c: char| "()[]{}".contains(c));
    let mut run = String::new();
    let mut rest = String::new();
    for c in part.chars().map(|c| if c == '\u{2217}' { '*' } else { c }) {
      if is_symbol_char(c) {
        if !rest.is_empty() {
          labels.push(std::mem::take(&mut rest));
        }
        if run.chars().next().is_some_and(|r| r != c) {
          labels.push(std::mem::take(&mut run));
        }
        run.push(c);
      } else {
        if !run.is_empty() {
          labels.push(std::mem::take(&mut run));
        }
        rest.push(c);
      }
    }
    labels.extend([run, rest].into_iter().filter(|s| !s.is_empty()));
  }
  labels
}

/// A label without its prefix (`affiliation:1` "1", `fn:label1` "label1"), Perl's `s/^[^:]*://`; `None` when it has
/// none.
pub(super) fn unprefixed(label: &str) -> Option<&str> {
  label.split_once(':').map(|(_, rest)| rest)
}

/// The creators a pending entry's label names, as the kernel relocates an annotation (Perl Base_Utility.pool.ltxml:
/// 885-900, `relocate.rs`): the creators requesting it exactly; else — misuse of labelling macros can lead to a prefix
/// mismatch — those whose request without its prefix is it, those requesting it without its prefix, those whose
/// request and it agree without both prefixes, the first that finds any (elsarticle's `\fnref{label1}`, `fn:label1`,
/// answers `\address[label1]`: 1712.00945; `\inst{1}`'s `affiliation:1` the entry `1`). A creator's own role-sequence
/// label `author:N` (the N-th author, `\lx@add@frontmatter@now`'s `_annotations`) is matched exactly and never by
/// that fallback (OD #159, relocate.rs:42-50: a shared `affiliation:1` must not reach the first author by number).
pub(super) fn label_owners(creators: &[RCreator], label: &str) -> Vec<usize> {
  if let Some(n) = label.strip_prefix("author:") {
    return n
      .trim()
      .parse::<usize>()
      .ok()
      .and_then(|n| n.checked_sub(1))
      .filter(|at| *at < creators.len())
      .into_iter()
      .collect();
  }
  let find = |matches: &dyn Fn(&str) -> bool| -> Vec<usize> {
    (0..creators.len())
      .filter(|&c| creators[c].requests.iter().any(|r| matches(r)))
      .collect()
  };
  let bare = unprefixed(label);
  let tiers: [&dyn Fn(&str) -> bool; 4] = [
    &|r| r == label,
    &|r| unprefixed(r) == Some(label),
    &|r| bare == Some(r),
    &|r| bare.is_some() && unprefixed(r) == bare,
  ];
  tiers
    .iter()
    .map(|tier| find(*tier))
    .find(|owners| !owners.is_empty())
    .unwrap_or_default()
}

/// Whether a request answers a label by any of [`label_owners`]'s tiers.
pub(super) fn label_matches(request: &str, label: &str) -> bool {
  request == label
    || unprefixed(request) == Some(label)
    || unprefixed(label) == Some(request)
    || unprefixed(label).is_some_and(|bare| unprefixed(request) == Some(bare))
}

/// A name's letters, lowercased, accents folded (`René` "rene", `Błaszak` "blaszak"): what a fuzzy label and a name
/// are compared by.
pub(super) fn fold_name(text: &str) -> String {
  text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .map(|c| match c {
      'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
      'ç' | 'ć' | 'č' => 'c',
      'ď' | 'đ' => 'd',
      'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
      'ğ' => 'g',
      'ì' | 'í' | 'î' | 'ï' | 'ı' => 'i',
      'ł' | 'ľ' => 'l',
      'ñ' | 'ń' | 'ň' => 'n',
      'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ő' => 'o',
      'ř' => 'r',
      'ś' | 'š' | 'ş' | 'ș' => 's',
      'ť' | 'ț' => 't',
      'ù' | 'ú' | 'û' | 'ü' | 'ů' | 'ű' => 'u',
      'ý' | 'ÿ' => 'y',
      'ź' | 'ż' | 'ž' => 'z',
      c => c,
    })
    .collect()
}

/// The letters a label written in TeX source spells (`Jean-Ren\'e Chazottes`): accent commands dropped, `\l`, `\o`
/// their letters, braces and ties aside, any other command's name dropped (its argument kept: `\textbf{Ann}`); the marks
/// a name's source carries dropped with their arguments — a mark or note command's (`\inst{1}`, `\thanks{…}`), a
/// formula (`$^{1}$`) — so `Luca Passalacqua (\Envelope)` spells "Luca Passalacqua" (0807.3898).
pub(super) fn tex_letters(source: &str) -> String {
  let mut out = String::new();
  let mut chars = source.chars().peekable();
  // skip a balanced group whose `{` is next
  fn skip_group(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    if chars.peek() != Some(&'{') {
      return;
    }
    let mut depth = 0i32;
    for c in chars.by_ref() {
      match c {
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            return;
          }
        },
        _ => {},
      }
    }
  }
  while let Some(c) = chars.next() {
    match c {
      '$' => {
        for m in chars.by_ref() {
          if m == '$' {
            break;
          }
        }
        continue;
      },
      '\\' => {},
      c => {
        out.push(if c == '~' { ' ' } else { c });
        continue;
      },
    }
    let mut word = String::new();
    while let Some(&n) = chars.peek() {
      if n.is_ascii_alphabetic() || n == '@' {
        word.push(n);
        chars.next();
      } else {
        break;
      }
    }
    if word.contains('@') {
      // (an internal command, its argument with it: what a symbol macro writes out to, `\Envelope` as
      // `\@chooseSymbol{'014}` once `ledger::label_text` expands the label, 0807.3898 — the digits were read as letters)
      while chars.peek() == Some(&' ') {
        chars.next();
      }
      skip_group(&mut chars);
    } else if word.is_empty() {
      // (an accent command, `\'`, `\"`, `\~`: its character dropped, the letter after it kept)
      chars.next();
    } else if let Some(rest) = word.strip_prefix("lxaccent") {
      // (an accent written out as the kernel's inner command, `\lxaccentumlaut`, `\lxaccentc` — `accent_inner_name`,
      // setup_binding_language.rs —, the letters glued after its name the label's: `Gizem \lxaccentc{S}eng\lxaccentumlautor`
      // spells "Gizem Sengor", 2205.11550)
      let tail = ["acute", "dot", "macron", "umlaut", "hat", "grave", "tilde"]
        .iter()
        .find_map(|name| rest.strip_prefix(name))
        .unwrap_or_else(|| rest.get(1..).unwrap_or(""));
      out.push_str(tail);
    } else if matches!(
      word.as_str(),
      "l" | "o" | "ss" | "ae" | "oe" | "aa" | "i" | "j" | "L" | "O"
    ) {
      out.push_str(&word);
    } else if matches!(
      word.as_str(),
      "inst"
        | "thanks"
        | "footnote"
        | "footnotemark"
        | "textsuperscript"
        | "email"
        | "orcid"
        | "orcidlink"
        | "label"
        | "ref"
    ) {
      // (a mark or a note of the name, with its argument)
      while chars.peek() == Some(&' ') {
        chars.next();
      }
      skip_group(&mut chars);
    }
  }
  out
}

/// The labels of the Mark a note's text opens with (the lexer's leading Mark of its body, `events.rs`: `$^{1}$Unit…`,
/// `\textsuperscript{a}`, a footnote symbol `*Corresponding author`), and its text without that mark — never read
/// from the flattened text, where `$^{1}$Unit` is the word "1Unit" (review revfms2a2 finding 5; 1909.02288).
pub(super) fn leading_mark(note: &Event) -> Option<(Vec<String>, String)> {
  let mark = note
    .inner
    .iter()
    .find(|e| e.kind == EventKind::Mark && e.line_first)?;
  let labels = mark_labels(mark.label.as_deref().unwrap_or(""));
  if labels.is_empty() {
    return None;
  }
  let text = note.text.trim_start();
  let legend = text.strip_prefix(mark.text.as_str()).unwrap_or(text);
  Some((labels, legend.trim().to_string()))
}

/// Whether a later line of a column whose first line is names is more names (review revfms2a2 finding 6: by positive
/// evidence, the text test only beside it — principled before heuristics). The evidence: the line follows a blank line
/// (`\\ \\`, a new author's lines: 0802.1860), or a names row it continues (the row before ended in a comma or "and":
/// `Juan Sebastian~Gomez-Diaz, and \newline Alejandro~Alvarez-Melcon`, 1403.0130), or the names row above has its
/// affiliation line already and this line is set in a font of its own against it (`Kei Harada \\ {\it Nagoya
/// University} \\ Hayato Saigo \\ …`: 1009.1429, 1108.3744; the same font, "Aptex Software" under "Chief Scientist",
/// 0803.3838, is the affiliation's) or comes after the row's labelled lines (`$^\dagger$Univ A \quad $^\ddagger$Univ B
/// \\ Cat Cole \\ Physics`, SYNC row 46), or its names show marks or notes (`Ann Able$^{1}$ Bob Baker$^{2}$`, `Irina Gheorghiciuc\thanks{…}`,
/// 1403.5246: a Mark or a Note right after a Name), or it is a row of two or more name pieces parted
/// by separators (`Triantafyllos Afouras\quad\quad Himel Chowdhury …` under the first row, 2111.03635), none holding
/// an acronym (`IBM India, IBM CIO`, 2609.10596: institutions); and then each
/// of its pieces must read as a name (the classify helpers). One piece with no evidence — "Capital Fund Management",
/// "Google DeepMind" (2606.07382, 2307.02318, 2305.16559, 2609.09054) — is the names' affiliation line, whatever its
/// text reads as.
pub(super) fn later_line_is_names(
  line: &Line<'_>,
  continued: bool,
  under_affiliation: bool,
  after_labelled: bool,
) -> bool {
  let events = &line.events;
  let marked = events.windows(2).any(|pair| {
    pair[0].kind == EventKind::Name
      && (pair[1].kind == EventKind::Mark && !pair[1].line_first || pair[1].kind == EventKind::Note)
  });
  let names: Vec<&&Event> = events
    .iter()
    .filter(|e| e.kind == EventKind::Name && e.text.chars().any(char::is_alphanumeric))
    .collect();
  let pieces = names.iter().filter(|e| !is_name_qualifier(&e.text)).count();
  // (the line's first run in a font of its own, against the affiliation line above it: the print sets the name apart)
  let own_font = events
    .iter()
    .take_while(|e| !inked(e))
    .any(|e| e.kind == EventKind::Layout && e.role.as_deref() == Some("font"));
  // (a row whose pieces each hold an acronym — a word of two or more capitals, no roman numeral — reads as
  // institutions, and its pieces alone are no evidence: `Bhanwar Gupta \\ IBM India, IBM CIO \\ …`, 2609.10596)
  let acronym = |word: &str| {
    let word = word.trim_matches(|c: char| !c.is_alphanumeric());
    word.chars().count() >= 2
      && word.chars().all(|c| c.is_ascii_uppercase())
      && !word.chars().all(|c| "IVXLC".contains(c))
  };
  // (in a piece not set in capitals: `MARTHA D. FERREIRA`'s words are no acronyms, 2202.13867)
  let institutions = names
    .iter()
    .filter(|e| !is_name_qualifier(&e.text))
    .any(|e| e.text.chars().any(char::is_lowercase) && e.text.split_whitespace().any(acronym));
  let evidence = line.after_blank
    || continued
    || (under_affiliation && (own_font || after_labelled))
    || marked
    || (pieces >= 2 && !institutions);
  evidence
    && !names.is_empty()
    && names
      .iter()
      .all(|e| reads_as_names(&e.text) || is_name_qualifier(&e.text))
}

/// Whether a labelled entry's text is a legend (a note on its authors) rather than their affiliation: what
/// `classify_thanks` reads as correspondence, a contribution, an address change or funding — funding only when the text
/// does not read as an institution (an institution named for its funder is an affiliation; before batch 78u's
/// whole-word match `classify_thanks`'s "erc" test also matched inside "Perception": `2 Guangdong Laboratory of Machine
/// Perception and Intelligent Computing`, 2509.05368).
pub(super) fn is_legend_text(text: &str) -> bool {
  match classify_thanks(text) {
    "note" => false,
    "funding" => !reads_as_affiliation(&text_tokens(text)),
    _ => true,
  }
}

/// Whether a label is a footnote symbol (ruling 2).
pub(super) fn is_symbol_label(label: &str) -> bool {
  !label.is_empty() && label.chars().all(is_symbol_char)
}

/// The text of events, as printed: names and contacts, a comma's `,`, a word's word, a gap's space; marks aside.
pub(super) fn events_text(events: &[&Event]) -> String {
  let mut out = String::new();
  for event in events {
    let piece = match &event.kind {
      EventKind::Name | EventKind::Contact => event.text.as_str(),
      EventKind::Sep(SepKind::Comma) => ",",
      EventKind::Sep(SepKind::Word(word)) => word.as_str(),
      _ => continue,
    };
    if piece == "," {
      out.push(',');
    } else {
      if !out.is_empty() {
        out.push(' ');
      }
      out.push_str(piece);
    }
  }
  out.trim().to_string()
}

/// `text` as tokens for the classify helpers (letters and others: `ExplodeText!`).
pub(super) fn text_tokens(text: &str) -> Tokens { Tokens::new(ExplodeText!(text)) }

/// The e-mail addresses printed in `text`, one per print (ruling 3); a braced group of local parts before a shared
/// domain written out, one each (`{takuma,ichise}@nii.ac.jp`, 1711.05435; the `\lx@split@addresses` reading, OD #805).
pub fn addresses_in(text: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = text;
  while let Some(open) = rest.find('{') {
    let Some(close) = rest[open..].find('}').map(|c| open + c) else {
      break;
    };
    let after = &rest[close + 1..];
    if let Some(domain) = after.strip_prefix('@') {
      // (the addresses printed before the group are the list's too: `wulijun3@mail2.sysu.edu.cn; stsljh@…;
      // \{fetia,taoqin,tyliu\}@microsoft.com`, 1808.08866 — they were lost)
      out.extend(plain_addresses(&rest[..open]));
      let domain: String = domain
        .chars()
        .take_while(|c| !c.is_whitespace() && !",;()<>{}".contains(*c))
        .collect();
      for local in rest[open + 1..close].split(',') {
        let local = local.trim();
        if !local.is_empty() && domain.contains('.') {
          out.push(format!("{local}@{}", domain.trim_end_matches('.')));
        }
      }
      rest = &after[1 + domain.len()..];
    } else {
      out.extend(plain_addresses(&rest[..=close]));
      rest = after;
    }
  }
  out.extend(plain_addresses(rest));
  out
}

/// The plain addresses of `text` (no braced group).
pub(super) fn plain_addresses(text: &str) -> Vec<String> {
  text
    .split(|c: char| c.is_whitespace() || ",;<>()[]{}".contains(c))
    .map(|w| {
      let w = w.trim_matches(|c: char| ".:".contains(c));
      // (a label glued to the address: `E-mail:lcai@cse.cuhk.edu.hk`, 1807.11339)
      let w = [
        "mailto:", "E-mail:", "e-mail:", "Email:", "email:", "E-Mail:", "EMail:",
      ]
      .iter()
      .find_map(|p| w.strip_prefix(p))
      .unwrap_or(w);
      w.trim_start_matches(':')
    })
    .filter(|w| {
      w.split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
    })
    .map(str::to_string)
    .collect()
}

/// A contact's role: the one it was given (an e-mail call's `email` even when a `mailto:` link inside it is a url, review
/// revfms2a2 finding 10, 2506.17875), `contact` when none; a url whose text is one e-mail address and nothing else is
/// an e-mail (`\url{ann@x.org}`, 1409.0496).
pub(super) fn contact_role(role: Option<&str>, text: &str) -> String {
  match role.filter(|r| !r.is_empty()) {
    Some("url") if is_one_address(text) => "email".into(),
    Some(role) => role.to_string(),
    None => "contact".into(),
  }
}

/// A Contact event's role and text: a link to an ORCID record is the creator's ORCID, its identifier the link's
/// (`\href{https://orcid.org/0000-0001-7461-3342}{\includegraphics{orcid}}`, 2307.02633; an ORCID logo drawn by TikZ
/// whose path is the link's text, 2304.11780, 1910.01653 — as the XML's `<ref href>` in the name); any other its
/// [`contact_role`] and its text.
pub(super) fn contact_parts(event: &Event) -> (String, String) {
  if let Some(id) = event
    .target
    .as_deref()
    .and_then(|t| t.split_once("orcid.org/"))
    .map(|(_, id)| id.trim().trim_end_matches('/'))
    .filter(|id| !id.is_empty())
  {
    return ("orcid".into(), id.to_string());
  }
  (
    contact_role(event.role.as_deref(), &event.text),
    event.text.clone(),
  )
}

/// Whether `text` is one e-mail address alone (a `mailto:` before it aside).
pub(super) fn is_one_address(text: &str) -> bool {
  let text = text.trim().trim_start_matches("mailto:");
  matches!(plain_addresses(text).as_slice(), [one] if one == text)
}

/// Whether `text` holds nothing but e-mail addresses (`{ann,bob}@x.org` one of them; an "E-mail:" before them,
/// separators between).
pub(super) fn only_addresses(text: &str) -> bool {
  // (a braced group of local parts before `@` is its address's)
  let mut flat = String::new();
  let mut rest = text;
  while let Some(open) = rest.find('{') {
    let Some(close) = rest[open..].find('}').map(|c| open + c) else {
      break;
    };
    flat.push_str(&rest[..open]);
    if !rest[close + 1..].starts_with('@') {
      flat.push_str(&rest[open..=close]);
    }
    rest = &rest[close + 1..];
  }
  flat.push_str(rest);
  flat
    .split(|c: char| c.is_whitespace() || ",;".contains(c))
    .filter(|w| !w.contains('@'))
    .map(|w| w.to_lowercase())
    .filter(|w| {
      !matches!(
        w.trim_matches(|c: char| !c.is_alphanumeric()),
        "e-mail" | "email" | "emails" | "e-mails" | "mail" | "and" | ""
      )
    })
    .all(|w| !w.chars().any(char::is_alphanumeric))
}

/// The creators a fuzzy label's names name by their surnames when the label spells no name whole (today's
/// OXIDIZED_DESIGN_DIVERGENCES #159 reading, relocate.rs: svjour3's `\institute{B. Wickramanayake \at …}` for "Bemali
/// Wickramanayake", 2011.14024; `Corresponding author: Guangming Zhu, Liang Zhang, and Syed Afaq Ali Shah \at …`,
/// 2106.12864; 2403.19676; `A.M.Bykov` 1205.2208): the label's names (commas, `&`, "and") each the one creator whose
/// surname (the last word) is its last word; none when a name names no one or several.
pub(super) fn surname_owners(creators: &[RCreator], letters: &str) -> Vec<usize> {
  let surname = |name: &str| {
    name
      .rsplit(|c: char| !c.is_alphabetic())
      .find(|w| !w.is_empty())
      .map(fold_name)
  };
  let surnames: Vec<Option<String>> = creators.iter().map(|c| surname(&c.name)).collect();
  let mut owners = Vec::new();
  for piece in letters
    .split([',', '&'])
    .flat_map(|p| p.split(" and "))
    .map(|p| p.trim().trim_start_matches("and ").trim())
    .filter(|p| !p.is_empty())
  {
    let key = surname(piece);
    let named: Vec<usize> = (0..creators.len())
      .filter(|&c| key.is_some() && surnames[c] == key)
      .collect();
    match named.as_slice() {
      [one] => {
        if !owners.contains(one) {
          owners.push(*one);
        }
      },
      _ => return Vec::new(),
    }
  }
  owners
}

/// `label` as two or more labels of `known` written together, the fewest pieces (`12` → `1`, `2`); with `symbols`, a
/// run of characters that are no letter or digit may stand alone too (`1⋆` → `1`, `⋆`), as long as one piece is known.
pub(super) fn parted_label(label: &str, known: &[String], symbols: bool) -> Option<Vec<String>> {
  let chars: Vec<(usize, char)> = label.char_indices().collect();
  let n = chars.len();
  if n < 2 {
    return None;
  }
  let at = |i: usize| chars.get(i).map_or(label.len(), |(b, _)| *b);
  // best[i]: the fewest pieces covering label[..i], with how many are known, and the piece before
  let mut best: Vec<Option<(usize, usize, usize)>> = vec![None; n + 1];
  best[0] = Some((0, 0, 0));
  for i in 0..n {
    let Some((count, hits, _)) = best[i] else {
      continue;
    };
    for j in i + 1..=n {
      let piece = &label[at(i)..at(j)];
      let is_known = known.iter().any(|k| k == piece);
      let is_symbol = symbols && piece.chars().all(|c| !c.is_alphanumeric());
      if !is_known && !is_symbol {
        continue;
      }
      let candidate = (count + 1, hits + usize::from(is_known), i);
      if best[j].is_none_or(|(c, h, _)| {
        (candidate.0, std::cmp::Reverse(candidate.1)) < (c, std::cmp::Reverse(h))
      }) {
        best[j] = Some(candidate);
      }
    }
  }
  let (count, hits, _) = best[n]?;
  if count < 2 || hits == 0 {
    return None;
  }
  let mut parts = Vec::new();
  let mut j = n;
  while j > 0 {
    let (_, _, i) = best[j]?;
    parts.push(label[at(i)..at(j)].to_string());
    j = i;
  }
  parts.reverse();
  Some(parts)
}

/// Whether an e-mail address is a pattern for the authors' addresses, not one of them: its local part is only the words
/// of a name template (`FirstName.LastName@SIT.Fraunhofer.de`, `{firstname.lastname}@…`, `name.surname@…`; 2006.12418).
pub(super) fn is_pattern_address(address: &str) -> bool {
  let Some((local, _)) = address.split_once('@') else {
    return false;
  };
  let words: Vec<String> = local
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect();
  !words.is_empty()
    && words.iter().all(|w| {
      matches!(
        w.as_str(),
        "first"
          | "last"
          | "firstname"
          | "lastname"
          | "name"
          | "surname"
          | "givenname"
          | "familyname"
          | "given"
          | "family"
          | "initial"
          | "initials"
          | "fname"
          | "lname"
      )
    })
}
