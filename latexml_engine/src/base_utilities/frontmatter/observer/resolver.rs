//! The observer's resolver (Layer C of the design note's §2; OXIDIZED_DESIGN_DIVERGENCES #837): a document's ledger —
//! each author block's events and the frontmatter API calls after them — read into the tuples `ab_audit`'s `front.rs`
//! extracts from the XML (each creator's name, affiliations, contacts and notes, and the author block's own), each
//! association with the rule that decided it. Class-independent; a family's conventions come from its policy entry
//! (`policy.rs`, Layer B). In SHADOW mode only (stage S2a.2): nothing it decides reaches the output.
//!
//! The rules, in order (design §2 points 1-9, the FIDELITY_CHARTER §4 rulings of 2026-10-21):
//! 1. **Geometry**: columns from `Sep(And)` (the `\and` family) and alignment tabs, unless the policy reads `\and` as
//!    inline (llncs); rows from `Break`.
//! 2. **Line type**: a line whose first ink is a mark is a labelled line, unless it is the block's first line and its
//!    entries read as names with their marks answered below (the prefix convention); a column's first line, or one
//!    reading as names, is a names line; a line of contacts only a contact line; any other an unmarked line, the
//!    classify helpers (`name_count`, `reads_as_affiliation`) deciding.
//! 3. **Names**: a names line split at its separators; each mark attached to the name beside it — the one before,
//!    or the one after a comma the mark is not glued to (a prefix), or the one after when no name precedes; an `\inst`
//!    after a comma the one before under the policy (OD #807). A font switch inside a name run keeps it whole (the
//!    lexer's run: gretsi's `first \textsc{last}`, 2508.03934).
//! 4. **Ruling 1**: the marks shown on a name are its requests (`1,2` → 1, 2; a symbol run glued to a label, `*1`, two
//!    requests); the name is its text without them. A labelled entry goes to every name requesting its label.
//! 5. **Ruling 2**: a line led by a footnote-symbol mark labels that line for every name carrying the same character;
//!    a symbol no name carries leaves its line to the block.
//! 6. **Unlabelled lines** go to the names row above them in their column (R12; llncs's `\\` line, OD #792).
//! 7. **Contacts**: own mark → spelled author → (the class's list in author order, `SpelledElseOrder`) → the group's
//!    only author → elimination (the one address no name spells, the one name no address spells: today's, OD #52 under
//!    user ruling R10) → the one author marked as the corresponding author → the block.
//! 8. **Ruling 3**: one contact per print event, nothing deduplicated (a line's addresses one each).
//! 9. **Ruling 4**: a note whose content all became contacts is dropped; an empty `\thanks{}` is kept.
//! 10. **API calls** after a block (not made by its expansion), creators and creator annotations only
//!     ([`creator_call`]): a creator; a labelled contact (its `label=`, its `labelseq=` place, its leading mark, or a
//!     `\label` a `\ref` mark names) to the names requesting it — a creator's own `author:N` exactly, any other label by
//!     Perl's prefix chain ([`label_owners`]); an unlabelled one by its `annotate=` (the last author, all, the run of new
//!     ones), authblk's by its `\AB@thenote` when the marks show.
//!
//! S2a.2 r2 (review revfms2a2) adds: names lines by evidence (`later_line_is_names`), a note's whole text and its
//! legend by its leading Mark, marks mid-line and trailing, today's joined unmarked lines and contact grid, a replacing
//! `\author` call (OXIDIZED_DESIGN_DIVERGENCES #837's r2 paragraph).

use super::{
  events::{Event, EventKind, SepKind},
  policy::{EmailOwner, Policy, policy_for},
  *,
};

/// One association the resolver made: a contact, an affiliation or a note, with its text, its label and the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assoc {
  /// `affiliation`, `email`, `url`, … for a contact; the note's role (`thanks`, `legend`) for a note
  pub role:   String,
  pub text:   String,
  pub label:  Option<String>,
  pub rule:   String,
  /// the event it came from: (ledger index of its block or API call, its span) — spans are unique only within a
  /// block (notes decision 4)
  pub source: Option<(usize, Option<(u16, u16)>)>,
}

/// One resolved creator.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RCreator {
  /// the name without its marks (ruling 1's fuzzy label)
  pub name:         String,
  /// the marks shown on it, its requests
  pub requests:     Vec<String>,
  /// how the name was split out
  pub rule:         String,
  /// the ledger index of its block (or API call)
  pub source:       usize,
  pub affiliations: Vec<Assoc>,
  pub contacts:     Vec<Assoc>,
  pub notes:        Vec<Assoc>,
}

/// A document's resolution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolved {
  pub creators: Vec<RCreator>,
  /// what no creator owns: the author block's own (a trailing nameless creator in the XML, OD #159)
  pub block:    Vec<Assoc>,
  /// what the rulings dropped: a note whose content all became contacts (ruling 4), with its text — kept for the
  /// comparison, which reads today's note there as the ruling's change
  pub dropped:  Vec<Assoc>,
  /// per block (ledger index): the policy entry it ran under
  pub policies: Vec<(usize, &'static str)>,
}

/// A resolver input: an author block (its events as the lexer gave them) or an API call made apart from the blocks.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
  Block {
    index:   usize,
    binding: String,
    hook:    String,
    /// the keyvals the call hands each author (`annotations={a,b}`: requests)
    keyvals: Option<String>,
    /// the policy's state counter at the call (`ledger::AuthorBlock::counter`)
    counter: Option<i64>,
    /// whether the call replaces the authors queued before it (`\lx@clear@frontmatter{ltx:creator}`: the kernel
    /// `\author` given twice), and the flushes before it (the queue it is in)
    replace: bool,
    epoch:   usize,
    events:  Vec<Event>,
  },
  Api {
    index:  usize,
    tag:    String,
    role:   String,
    attrs:  String,
    events: Vec<Event>,
  },
}

/// `key=value` of an attribute string (`role=affiliation,label=a`), braces stripped.
fn attr_value<'a>(attrs: &'a str, key: &str) -> Option<&'a str> {
  split_top_commas(attrs).into_iter().find_map(|kv| {
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
fn split_top_commas(text: &str) -> Vec<&str> {
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
fn is_symbol_char(c: char) -> bool {
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
fn unprefixed(label: &str) -> Option<&str> { label.split_once(':').map(|(_, rest)| rest) }

/// The creators a pending entry's label names, as the kernel relocates an annotation (Perl Base_Utility.pool.ltxml:
/// 885-900, `relocate.rs`): the creators requesting it exactly; else — misuse of labelling macros can lead to a prefix
/// mismatch — those whose request without its prefix is it, those requesting it without its prefix, those whose
/// request and it agree without both prefixes, the first that finds any (elsarticle's `\fnref{label1}`, `fn:label1`,
/// answers `\address[label1]`: 1712.00945; `\inst{1}`'s `affiliation:1` the entry `1`). A creator's own role-sequence
/// label `author:N` (the N-th author, `\lx@add@frontmatter@now`'s `_annotations`) is matched exactly and never by
/// that fallback (OD #159, relocate.rs:42-50: a shared `affiliation:1` must not reach the first author by number).
fn label_owners(creators: &[RCreator], label: &str) -> Vec<usize> {
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
fn label_matches(request: &str, label: &str) -> bool {
  request == label
    || unprefixed(request) == Some(label)
    || unprefixed(label) == Some(request)
    || unprefixed(label).is_some_and(|bare| unprefixed(request) == Some(bare))
}

/// A name's letters, lowercased, accents folded (`René` "rene", `Błaszak` "blaszak"): what a fuzzy label and a name
/// are compared by.
fn fold_name(text: &str) -> String {
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
fn tex_letters(source: &str) -> String {
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
      if n.is_ascii_alphabetic() {
        word.push(n);
        chars.next();
      } else {
        break;
      }
    }
    if word.is_empty() {
      // (an accent command, `\'`, `\"`, `\~`: its character dropped, the letter after it kept)
      chars.next();
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
fn leading_mark(note: &Event) -> Option<(Vec<String>, String)> {
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
/// by separators (`Triantafyllos Afouras\quad\quad Himel Chowdhury …` under the first row, 2111.03635); and then each
/// of its pieces must read as a name (the classify helpers). One piece with no evidence — "Capital Fund Management",
/// "Google DeepMind" (2606.07382, 2307.02318, 2305.16559, 2609.09054) — is the names' affiliation line, whatever its
/// text reads as.
fn later_line_is_names(
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
  let evidence = line.after_blank
    || continued
    || (under_affiliation && (own_font || after_labelled))
    || marked
    || pieces >= 2;
  evidence
    && !names.is_empty()
    && names
      .iter()
      .all(|e| reads_as_names(&e.text) || is_name_qualifier(&e.text))
}

/// Whether a labelled entry's text is a legend (a note on its authors) rather than their affiliation: what
/// `classify_thanks` reads as correspondence, a contribution, an address change or funding — funding only when the text
/// does not read as an institution (its "erc" test matches "Perception": `2 Guangdong Laboratory of Machine Perception
/// and Intelligent Computing`, 2509.05368).
fn is_legend_text(text: &str) -> bool {
  match classify_thanks(text) {
    "note" => false,
    "funding" => !reads_as_affiliation(&text_tokens(text)),
    _ => true,
  }
}

/// Whether a label is a footnote symbol (ruling 2).
fn is_symbol_label(label: &str) -> bool { !label.is_empty() && label.chars().all(is_symbol_char) }

/// The text of events, as printed: names and contacts, a comma's `,`, a word's word, a gap's space; marks aside.
fn events_text(events: &[&Event]) -> String {
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
fn text_tokens(text: &str) -> Tokens { Tokens::new(ExplodeText!(text)) }

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
fn plain_addresses(text: &str) -> Vec<String> {
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
fn contact_role(role: Option<&str>, text: &str) -> String {
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
fn contact_parts(event: &Event) -> (String, String) {
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
fn is_one_address(text: &str) -> bool {
  let text = text.trim().trim_start_matches("mailto:");
  matches!(plain_addresses(text).as_slice(), [one] if one == text)
}

/// Whether `text` holds nothing but e-mail addresses (`{ann,bob}@x.org` one of them; an "E-mail:" before them,
/// separators between).
fn only_addresses(text: &str) -> bool {
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

/// A line of a block: its column, its events, and whether a blank line (a row of no ink) comes before it in its column.
struct Line<'a> {
  column:      usize,
  events:      Vec<&'a Event>,
  after_blank: bool,
}

/// A labelled entry of a labelled line: its labels, its events, whether a symbol leads it.
struct Entry<'a> {
  labels: Vec<String>,
  symbol: bool,
  events: Vec<&'a Event>,
}

/// The block's lines (rule 1): a `Break` ends a row, a column separator (unless `\and` is inline) a column.
fn lines_of<'a>(events: &'a [Event], and_inline: bool) -> Vec<Line<'a>> {
  let new = |column| Line {
    column,
    events: Vec::new(),
    after_blank: false,
  };
  let mut lines = vec![new(0)];
  let mut column = 0usize;
  for event in events {
    match &event.kind {
      EventKind::Break => lines.push(new(column)),
      EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) if !and_inline => {
        column += 1;
        lines.push(new(column));
      },
      EventKind::ApiCall => {},
      _ => lines.last_mut().expect("a line").events.push(event),
    }
  }
  // (a row of no ink between two inked rows of a column is a blank line)
  let mut inked_before: Vec<usize> = Vec::new();
  let mut blank = false;
  for line in &mut lines {
    let has_ink = line.events.iter().any(|e| inked(e));
    if !has_ink {
      blank = inked_before.contains(&line.column);
      continue;
    }
    line.after_blank = blank && inked_before.contains(&line.column);
    blank = false;
    inked_before.push(line.column);
  }
  lines.retain(|line| line.events.iter().any(|e| inked(e)));
  lines
}

/// Whether an event is ink a line's type is read from (not layout, not a separator).
fn inked(event: &Event) -> bool {
  matches!(
    event.kind,
    EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note
  )
}

/// The block's events with an IEEE template's ordinal before a name (`1\textsuperscript{st} Sertac Kilickaya`,
/// 2412.10792, 2202.06596) one Name with it, as today's personname "1 Sertac Kilickaya": the ordinal's superscript is
/// no request.
fn join_ordinals(events: &[Event]) -> Vec<Event> {
  let mut out: Vec<Event> = Vec::with_capacity(events.len());
  let mut i = 0;
  while i < events.len() {
    let event = &events[i];
    if event.kind == EventKind::Name
      && !event.text.is_empty()
      && event.text.chars().all(|c| c.is_ascii_digit())
      && let Some(mark) = events.get(i + 1)
      && mark.kind == EventKind::Mark
      && matches!(mark.label.as_deref(), Some("st" | "nd" | "rd" | "th"))
      && let Some(name) = events.get(i + 2).filter(|e| e.kind == EventKind::Name)
    {
      out.push(Event {
        text: s!("{} {}", event.text, name.text),
        span: match (event.span, name.span) {
          (Some((a, _)), Some((_, b))) => Some((a, b)),
          (a, b) => a.or(b),
        },
        ..name.clone()
      });
      i += 3;
      continue;
    }
    out.push(event.clone());
    i += 1;
  }
  out
}

/// The block's events with a mark printed as text where a line opens: a Name whose first word is a symbol some name
/// shows as its mark (`\small\em$\sharp$ - CBCL, …` under `Mroueh$^{\sharp}$`, 1209.1360 — `♯` is no `\@fnsymbol`
/// character, so the lexer read it as ink) parted into that Mark and the rest.
fn split_text_marks(events: &[Event]) -> Vec<Event> {
  let shown: Vec<String> = events
    .iter()
    .filter(|e| e.kind == EventKind::Mark && !e.line_first)
    .flat_map(|e| mark_labels(e.label.as_deref().unwrap_or("")))
    .filter(|l| !l.chars().any(char::is_alphanumeric))
    .collect();
  if shown.is_empty() {
    return events.to_vec();
  }
  let mut out = Vec::with_capacity(events.len());
  let mut line_start = true;
  for event in events {
    match event.kind {
      EventKind::Break | EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => line_start = true,
      EventKind::Name if line_start => {
        line_start = false;
        let (first, rest) = event
          .text
          .split_once(' ')
          .unwrap_or((event.text.as_str(), ""));
        if shown.iter().any(|l| l == first) {
          let mut mark = Event::new(EventKind::Mark);
          mark.label = Some(first.to_string());
          mark.text = first.to_string();
          mark.symbol = true;
          mark.line_first = true;
          mark.role = Some("text".to_string());
          mark.span = event.span;
          mark.nest = event.nest;
          out.push(mark);
          let rest = rest.trim_start_matches(|c: char| c == '-' || c.is_whitespace());
          if !rest.is_empty() {
            out.push(Event {
              text: rest.to_string(),
              ..event.clone()
            });
          }
          continue;
        }
      },
      EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note => {
        line_start = false
      },
      _ => {},
    }
    out.push(event.clone());
  }
  out
}

/// The labelled entries of a line whose marks trail their text (`Nanyang Technological University$^1$, Amazon$^2$`,
/// 2507.05750): each mark after a Name closes an entry of the ink since the entry before; the line must end in a mark.
fn trailing_entries<'a>(line: &[&'a Event]) -> Vec<Entry<'a>> {
  let mut entries: Vec<Entry<'a>> = Vec::new();
  let mut run: Vec<&'a Event> = Vec::new();
  let mut closed = false;
  for event in line {
    match &event.kind {
      EventKind::Mark => {
        let labels = mark_labels(event.label.as_deref().unwrap_or(""));
        if run.iter().any(|e| e.kind == EventKind::Name) {
          while run
            .last()
            .is_some_and(|e| matches!(e.kind, EventKind::Sep(SepKind::Comma)))
          {
            run.pop();
          }
          let symbol = event.symbol;
          entries.push(Entry {
            labels,
            symbol,
            events: std::mem::take(&mut run),
          });
          closed = true;
        } else if let Some(entry) = entries.last_mut().filter(|_| closed) {
          entry.labels.extend(labels);
        } else {
          return Vec::new();
        }
      },
      EventKind::Layout | EventKind::Sep(SepKind::Gap { .. }) => {},
      EventKind::Sep(SepKind::Comma) if run.is_empty() => {},
      _ => {
        closed = false;
        run.push(event);
      },
    }
  }
  if run.iter().any(|e| inked(e)) {
    return Vec::new();
  }
  entries
}

/// The labelled entries of a mark-led line: each opened by a mark at the line's start or after a separator (a gap, a
/// comma), and the ink up to the next such mark; a mark right after an entry's ink is shown on it (no label of it).
fn entries_of<'a>(line: &[&'a Event]) -> Vec<Entry<'a>> {
  let mut entries: Vec<Entry<'a>> = Vec::new();
  let mut after_sep = true;
  let mut last_ink: Option<&Event> = None;
  for event in line {
    match &event.kind {
      EventKind::Mark => {
        // (a mark set off from the ink before it by a space — the source positions between them — opens an entry of
        // its own: `$^1$Arizona State University $^2$Rider University`, 2601.20069; one glued to it is shown on it)
        if let (Some(prev), Some((start, _))) = (last_ink.and_then(|e| e.span), event.span)
          && start > prev.1
        {
          after_sep = true;
        }
        let labels = mark_labels(event.label.as_deref().unwrap_or(""));
        let symbol =
          event.symbol || (!labels.is_empty() && labels.iter().all(|l| is_symbol_label(l)));
        let open = entries.last().is_some_and(|e| e.events.is_empty());
        if open {
          if let Some(entry) = entries.last_mut() {
            entry.labels.extend(labels);
            entry.symbol &= symbol;
          }
        } else if after_sep || entries.is_empty() {
          entries.push(Entry {
            labels,
            symbol,
            events: Vec::new(),
          });
        }
        after_sep = false;
      },
      EventKind::Sep(SepKind::Gap { .. }) => after_sep = true,
      EventKind::Sep(SepKind::Comma) => {
        after_sep = true;
        if let Some(entry) = entries.last_mut() {
          entry.events.push(event);
        }
      },
      EventKind::Layout => {},
      // (punctuation opening an entry is no text of it: the `)` of `($^1$) Univ A`, 1004.4809)
      EventKind::Name
        if !event.text.chars().any(char::is_alphanumeric)
          && entries.last().is_some_and(|e| e.events.is_empty()) => {},
      _ => {
        after_sep = false;
        last_ink = Some(event);
        if let Some(entry) = entries.last_mut() {
          entry.events.push(event);
        }
      },
    }
    if event.kind == EventKind::Mark {
      last_ink = Some(event);
    }
  }
  let count = entries.len();
  for (i, entry) in entries.iter_mut().enumerate() {
    // (the separator before the next entry is no entry's: `Berkeley and $^2$National Bureau …`, 1810.07216; the line's
    // last entry keeps a word the next line continues, `Biocomputación and \\ Física …`, 1107.1186)
    while entry.events.last().is_some_and(|e| match e.kind {
      EventKind::Sep(SepKind::Comma) => true,
      EventKind::Sep(SepKind::Word(_)) => i + 1 < count,
      _ => false,
    }) {
      entry.events.pop();
    }
  }
  entries
}

/// Whether a text reads as names (the classify helpers: name-shaped parts, no institution word, no address).
fn reads_as_names(text: &str) -> bool {
  !text.contains('@') && name_count(text) > 0 && !reads_as_affiliation(&text_tokens(text))
}

/// The entry a line ended with, which an unmarked line under it continues: a labelled entry (its place in
/// `Resolver::pending`).
enum Continued {
  Pending(usize),
  /// an unmarked line's affiliation under a marked names row: each owner's (creator, its place among the affiliations)
  Joined(Vec<(usize, usize)>),
}

/// The resolution of a block's names line: its creators (indices into `creators`) and what it attached.
struct NamesLine {
  creators: Vec<usize>,
}

/// The state of a document's resolution.
struct Resolver {
  out:              Resolved,
  /// per creator: its block (input index) and the roles its contacts hold
  roles:            Vec<Vec<String>>,
  /// the labels of `labelseq=<role>` entries so far, per role
  sequence:         Vec<(String, usize)>,
  /// labelled entries of the blocks not yet answered (label, entry), resolved at the end against every request
  pending:          Vec<(Vec<String>, bool, Assoc)>,
  /// per creator: its group (the block's input index, or the policy's call group)
  group:            Vec<usize>,
  /// the addresses of a class's list printed apart from the names (`EmailOwner::SpelledElseOrder`), each with its
  /// group's creators
  /// (each with its call's place in the list: `_sequence=<n>`, else the calls' count)
  sequenced:        Vec<(String, Assoc, Vec<usize>, usize)>,
  /// the calls of the list so far
  sequence_calls:   usize,
  /// the last `annotate=run` contact's role, the creators then, and the run it reached
  last_run:         Option<(String, usize, Vec<usize>)>,
  /// per creator: its contacts in the order they were attached (an affiliation, by its place among the
  /// affiliations; else a contact's), for [`Resolver::distribute_upfront`]
  order:            Vec<Vec<(bool, usize)>>,
  /// authblk's notes (`Policy::thenote`): the state its `\author` and `\affil` keep
  thenote:          TheNote,
  /// the requests authblk's printed `\AB@thenote` made (creator, label): what ruling 1 reads off the PDF's marks
  thenote_requests: Vec<(usize, String)>,
  /// per creator of a block: that block's queue (its `epoch`); the creators a later block replaced
  epoch_of:         Vec<Option<usize>>,
  replaced:         Vec<usize>,
  /// a note's e-mails (the creator the note is on, the contact), placed once every name is read
  note_emails:      Vec<(Option<usize>, Assoc)>,
  /// the requests a glued mark was parted into (creator, request), `part_glued_labels`
  parted:           Vec<(usize, String)>,
  /// while an API call's marked entries are resolved one by one (`mark_led_pieces`)
  splitting:        bool,
}

/// authblk's `\AB@thenote` state (authblk.sty:114-118, 148-151), read in call order: the `affil` counter, stepped at
/// the first `\author` after an `\affil` (`\ifnewaffil`); whether any label was given (which sets `Maxaffil` to 0,
/// so the marks show); `\c@Maxaffil` at the last block's call; each unlabelled author's number; each unlabelled
/// `\affil`'s number, with the rule tag its run placement carries.
#[derive(Default)]
struct TheNote {
  counter:  usize,
  newaffil: bool,
  labelled: bool,
  maxaffil: Option<i64>,
  run_of:   Vec<(usize, usize)>,
  affils:   Vec<(usize, String)>,
}

impl Resolver {
  fn new_creator(
    &mut self,
    name: String,
    rule: &str,
    source: usize,
    requests: Vec<String>,
    group: usize,
  ) -> usize {
    self.out.creators.push(RCreator {
      name,
      requests,
      rule: rule.to_string(),
      source,
      ..RCreator::default()
    });
    self.roles.push(Vec::new());
    self.order.push(Vec::new());
    self.group.push(group);
    self.out.creators.len() - 1
  }

  fn attach(&mut self, owner: Option<usize>, assoc: Assoc, note: bool) {
    match owner {
      Some(at) => {
        let creator = &mut self.out.creators[at];
        if note {
          creator.notes.push(assoc);
        } else {
          self.roles[at].push(assoc.role.clone());
          if assoc.role == "affiliation" || assoc.role == "address" {
            self.order[at].push((true, creator.affiliations.len()));
            creator.affiliations.push(assoc);
          } else {
            self.order[at].push((false, creator.contacts.len()));
            creator.contacts.push(assoc);
          }
        }
      },
      None => self.out.block.push(assoc),
    }
  }

  /// The one candidate an address spells (`spelled_owners`), if one.
  fn spelled(&self, address: &str, candidates: &[usize]) -> Option<usize> {
    let keys: Vec<NameKeys> = candidates
      .iter()
      .map(|&c| NameKeys::of(&text_tokens(&self.out.creators[c].name)))
      .collect();
    match spelled_owners(&keys, address).as_slice() {
      [one] => Some(candidates[*one]),
      _ => None,
    }
  }

  /// The one candidate whose note names them the corresponding author, else the block.
  fn corresponding(&self, candidates: &[usize]) -> (Option<usize>, &'static str) {
    let corresponding: Vec<usize> = candidates
      .iter()
      .copied()
      .filter(|&c| {
        self.out.creators[c]
          .notes
          .iter()
          .any(|n| names_correspondence(&n.text))
      })
      .collect();
    match corresponding.as_slice() {
      [one] if candidates.len() > 1 => (Some(*one), "owner:corresponding"),
      _ => (None, "owner:block"),
    }
  }

  /// Contacts of `text` (one per printed address, ruling 3; the text whole when it holds none), owned by the chain:
  /// the spelled ones first, then — the one address left, and the one candidate left without an address — by
  /// elimination (R10), then the rest of the chain.
  fn place_line_contacts(
    &mut self,
    text: &str,
    role: &str,
    candidates: &[usize],
    source: (usize, Option<(u16, u16)>),
  ) {
    let addresses = addresses_in(text);
    if addresses.is_empty() {
      let (owner, rule) = match candidates {
        [only] => (Some(*only), "owner:only-author"),
        _ => (None, "owner:block"),
      };
      self.attach(
        owner,
        Assoc {
          role:   role.to_string(),
          text:   text.to_string(),
          label:  None,
          rule:   rule.into(),
          source: Some(source),
        },
        false,
      );
      return;
    }
    let spelled: Vec<Option<usize>> = addresses
      .iter()
      .map(|a| self.spelled(a, candidates))
      .collect();
    let unspelled = spelled.iter().filter(|s| s.is_none()).count();
    let left: Vec<usize> = candidates
      .iter()
      .copied()
      .filter(|c| !spelled.contains(&Some(*c)))
      .filter(|&c| {
        !self.out.creators[c]
          .contacts
          .iter()
          .any(|a| a.role == "email")
      })
      .collect();
    // (a list with one address per author that every spelled address agrees with is in author order: FIDELITY_CHARTER
    // §4, 2026-10-21, OD #805 — `{qkv, riven, jiyicheng.cs, lihuan.cs, cwang85}@zju.edu.cn` under five names, 2603.19610;
    // 2512.22629, 2305.12998, 2105.11174, 2412.09666)
    let in_order = addresses.len() == candidates.len()
      && candidates.len() >= 2
      && spelled
        .iter()
        .zip(candidates)
        .all(|(s, c)| s.is_none_or(|s| s == *c));
    for (at, (address, spelled)) in addresses.into_iter().zip(spelled).enumerate() {
      let (owner, rule) = match spelled {
        Some(owner) => (Some(owner), "owner:spelled"),
        None if in_order => (Some(candidates[at]), "owner:order"),
        None if unspelled == 1 && candidates.len() > 1 && left.len() == 1 => {
          (Some(left[0]), "owner:elimination")
        },
        None => match candidates {
          [only] => (Some(*only), "owner:only-author"),
          _ => self.corresponding(candidates),
        },
      };
      self.attach(
        owner,
        Assoc {
          role:   "email".into(),
          text:   address,
          label:  None,
          rule:   s!("{rule} ruling3:one-per-print"),
          source: Some(source),
        },
        false,
      );
    }
  }

  /// Resolve one block (its events under `policy`).
  fn block(
    &mut self,
    index: usize,
    policy: &'static Policy,
    cautious: bool,
    keyvals: Option<&str>,
    events: &[Event],
  ) {
    self.out.policies.push((index, policy.name));
    let and_inline = policy.and_inline;
    let events = &split_text_marks(&join_ordinals(events));
    let lines = lines_of(events, and_inline);
    let block_requests: Vec<String> = keyvals
      .and_then(|kv| attr_value(kv, "annotations"))
      .map(|labels| {
        labels
          .split(',')
          .map(|l| l.trim().to_string())
          .filter(|l| !l.is_empty())
          .collect()
      })
      .unwrap_or_default();
    let group = index;
    let block_creators_start = self.out.creators.len();
    // the labels the block's later mark-led lines answer (for the prefix convention)
    let answered_below: Vec<String> = lines
      .iter()
      .skip(1)
      .filter(|line| {
        line
          .events
          .iter()
          .find(|e| {
            inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
          })
          .is_some_and(|e| e.kind == EventKind::Mark)
      })
      .flat_map(|line| entries_of(&line.events).into_iter().flat_map(|e| e.labels))
      .collect();
    // per column: the creators of its last names row
    let mut row_above: Vec<(usize, Vec<usize>)> = Vec::new();
    // the entry the previous line ended with, which an unmarked line under it continues (an address's second line:
    // `$^1$Dept of X \\ 1-1-1 Street, City`, 0803.3589; one affiliation, as today's `\\`-joined contact)
    let mut continues: Option<Continued> = None;
    let mut first_in_column: Vec<usize> = Vec::new();
    // the columns whose last line is a names row ending in a separator (a comma, "and"), which the next line continues
    let mut open_rows: Vec<usize> = Vec::new();
    // the columns whose names row has a line under it already (its affiliation)
    let mut answered_rows: Vec<usize> = Vec::new();
    // the columns whose last line was labelled entries
    let mut labelled_last: Vec<usize> = Vec::new();
    // an unmarked line's affiliation whose group runs on past the line (its places, where the group ends)
    let mut open_group: Option<(u16, Vec<(usize, usize)>)> = None;
    for (n, line) in lines.iter().enumerate() {
      let continued = open_rows.contains(&line.column);
      open_rows.retain(|c| *c != line.column);
      // (the line before in the column was labelled entries: an unmarked name under them is a new row's — SYNC row 46,
      // `$^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics`)
      let after_labelled = labelled_last.contains(&line.column);
      labelled_last.retain(|c| *c != line.column);
      let under_affiliation = answered_rows.contains(&line.column);
      // (a line that is no names row answers the row above; a names row below takes this back)
      if !answered_rows.contains(&line.column) && row_above.iter().any(|(c, _)| *c == line.column) {
        answered_rows.push(line.column);
      }
      // (punctuation before a mark does not lead the line: `($^1$) LSIIT …`, 1004.4809)
      let first_ink = line
        .events
        .iter()
        .find(|e| {
          inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
        })
        .copied();
      let column_first = !first_in_column.contains(&line.column);
      if column_first {
        first_in_column.push(line.column);
      }
      let above = row_above
        .iter()
        .rev()
        .find(|(c, _)| *c == line.column)
        .map(|(_, r)| r.clone());
      let above = above
        .or_else(|| row_above.last().map(|(_, r)| r.clone()))
        .unwrap_or_default();
      let carried = continues.take();
      match first_ink.map(|e| &e.kind) {
        Some(EventKind::Mark) => {
          let entries = entries_of(&line.events);
          // (or the block is that one line, its affiliations the calls after it: revtex's `\author{$^{1}$Masashi Kimura
          // and $^{2,3}$Takahiro Tanaka}`, 1805.08625; 1405.2400, 2001.00788, 0710.4115, gr-qc0606119 — its names lost)
          let prefix = n == 0
            && !entries.is_empty()
            && entries.iter().all(|e| {
              reads_as_names(&events_text(&e.events))
                && (lines.len() == 1 || e.labels.iter().any(|l| answered_below.contains(l)))
            });
          if prefix {
            let mut row = Vec::new();
            for entry in entries {
              for name in names_of(&entry.events) {
                let c = self.new_creator(
                  name,
                  "line:names mark:prefix",
                  index,
                  [entry.labels.clone(), block_requests.clone()].concat(),
                  group,
                );
                row.push(c);
              }
            }
            answered_rows.retain(|c| *c != line.column);
            row_above.push((line.column, row));
            continue;
          }
          labelled_last.push(line.column);
          continues = self.labelled_entries(index, entries, "");
        },
        Some(_) => {
          let text = events_text(&line.events);
          let only_contacts = line
            .events
            .iter()
            .filter(|e| inked(e))
            .all(|e| e.kind == EventKind::Contact);
          // (a column's first line that is one link printing a name is that name: `\href{https://orcid.org/…}
          // {\includegraphics{orcid}\hspace{1mm}Amanda A. Howard}`, 2311.06483)
          if column_first && only_contacts {
            let links: Vec<&Event> = line
              .events
              .iter()
              .copied()
              .filter(|e| e.kind == EventKind::Contact)
              .collect();
            if let [link] = links.as_slice()
              && link.role.as_deref() == Some("url")
              && reads_as_names(&link.text)
            {
              let c = self.new_creator(
                link.text.clone(),
                "line:names link-text",
                index,
                block_requests.clone(),
                group,
              );
              answered_rows.retain(|c| *c != line.column);
              row_above.push((line.column, vec![c]));
              continue;
            }
          }
          // (a later line is more names when each of its pieces reads as a name: `Triantafyllos Afouras\quad\quad
          // Himel Chowdhury …` under the first row, 2111.03635; the classify helpers on each piece, not the run of them)
          // (a line of e-mail addresses alone is never names, a column's first line too: `\And
          // \{sagghosh, …, niranjan\}@cs.stonybrook.edu`, 2210.05991)
          let address_line = !addresses_in(&text).is_empty() && only_addresses(&text);
          let names_line = !only_contacts
            && !address_line
            && (column_first
              || later_line_is_names(line, continued, under_affiliation, after_labelled));
          if names_line {
            answered_rows.retain(|c| *c != line.column);
            if line
              .events
              .iter()
              .rev()
              .find(|e| {
                !matches!(
                  e.kind,
                  EventKind::Layout | EventKind::Contact | EventKind::Note
                )
              })
              .is_some_and(|e| matches!(e.kind, EventKind::Sep(SepKind::Comma | SepKind::Word(_))))
            {
              open_rows.push(line.column);
            }
            let made = self.names_line(index, policy, cautious, line, &block_requests, group);
            row_above.push((line.column, made.creators));
            continue;
          }
          if only_contacts {
            for contact in line.events.iter().filter(|e| e.kind == EventKind::Contact) {
              self.contact_event(index, contact, &above, "line:contact R12:row-above");
            }
            continue;
          }
          // (a line whose marks trail their text is labelled entries: `Nanyang Technological University$^1$,
          // Amazon$^2$`, 2507.05750)
          // (each of its marks one a name shows on another line: a symbol after an affiliation, `Harvard
          // University$^{*}$`, points to a note, 2307.02318)
          let trailing = trailing_entries(&line.events);
          let shown_elsewhere = |label: &String| {
            lines.iter().enumerate().any(|(m, other)| {
              m != n
                && other.events.iter().any(|e| {
                  e.kind == EventKind::Mark
                    && mark_labels(e.label.as_deref().unwrap_or("")).contains(label)
                })
            })
          };
          if !trailing.is_empty()
            && trailing
              .iter()
              .all(|e| e.labels.iter().any(shown_elsewhere))
          {
            continues = self.labelled_entries(index, trailing, " mark:trailing");
            continue;
          }
          // an unmarked line under names: an address line by the owner chain, else the row above's affiliation
          let source = (
            index,
            line.events.iter().find(|e| inked(e)).and_then(|e| e.span),
          );
          let notes: Vec<&Event> = line
            .events
            .iter()
            .copied()
            .filter(|e| e.kind == EventKind::Note)
            .collect();
          if text.is_empty() {
            for note in notes {
              self.note(index, note, above.last().copied(), "note:line-owner");
            }
            continue;
          }
          // (an unmarked line under a labelled entry continues it, as today's one `\\`-joined contact — a line holding an
          // address beside other text too: `$^{(1)}$Dept \\ Univ \\ City. \{ann,bob\}@upc.edu`, 2509.12185; a line of
          // addresses alone is e-mails by the owner chain, below: `$^1$Dept \\ landi@univ.trieste.it`, 0801.3376, 0810.3062)
          // (a line inside the group an unmarked line above opened is that affiliation's: `\\ \small{\textit{Dept …,
          // and\\ Institute …,\\ Facultad …}}`, one print unit, as today's one contact — 1012.1813)
          let starts = source.1.map(|(start, _)| start);
          match open_group.take() {
            Some((until, places)) if starts.is_some_and(|start| start < until) => {
              self.continue_entry(&Continued::Joined(places.clone()), &text);
              open_group = Some((until, places));
              continue;
            },
            _ => {},
          }
          if let Some(carried) =
            carried.filter(|_| addresses_in(&text).is_empty() || !only_addresses(&text))
          {
            self.continue_entry(&carried, &text);
            for note in notes {
              self.note(index, note, above.last().copied(), "note:line-owner");
            }
            continues = Some(carried);
            continue;
          }
          if !addresses_in(&text).is_empty() {
            let candidates: Vec<usize> = (block_creators_start..self.out.creators.len()).collect();
            self.place_line_contacts(&text, "email", &candidates, source);
            continue;
          }
          // (each unmarked line its own contact, as each `\\` line is today's: 0707.1113, 1404.0431; ruling 3 — but under a
          // names row whose names show marks no line answers, the lines are one affiliation of every name, as today's
          // `\\`-joined contact: `Riemann$^1$, … \\ 1- DESY in Zeuthen \\ Platanenallee 6, …`, 0903.2366, 2303.09899; a
          // footnote symbol is no such mark, `Marciniak$^{*}$ \\ Department … \\ Campus …` stays a contact a line,
          // 0707.1113)
          let joined = above.iter().any(|&c| {
            self.out.creators[c]
              .requests
              .iter()
              .any(|r| !is_symbol_label(r))
          });
          let mut places = Vec::new();
          for owner in &above {
            places.push((*owner, self.out.creators[*owner].affiliations.len()));
            self.attach(
              Some(*owner),
              Assoc {
                role:   "affiliation".into(),
                text:   text.clone(),
                label:  None,
                rule:   if joined {
                  "line:unmarked-joined R12:row-above".into()
                } else {
                  "line:unmarked R12:row-above".into()
                },
                source: Some(source),
              },
              false,
            );
          }
          let line_end = line
            .events
            .iter()
            .rev()
            .find_map(|e| e.span)
            .map(|(_, end)| end);
          let group_end = line
            .events
            .iter()
            .filter_map(|e| e.span)
            .map(|(_, end)| end)
            .max();
          if let (Some(line_end), Some(group_end)) = (line_end, group_end)
            && group_end > line_end
            && !places.is_empty()
          {
            open_group = Some((group_end, places.clone()));
          }
          if joined && !places.is_empty() {
            continues = Some(Continued::Joined(places));
          }
          if above.is_empty() {
            self.attach(
              None,
              Assoc {
                role:   "affiliation".into(),
                text:   text.clone(),
                label:  None,
                rule:   "line:unmarked owner:block".into(),
                source: Some(source),
              },
              false,
            );
          }
          for note in notes {
            self.note(index, note, above.last().copied(), "note:line-owner");
          }
        },
        None => {},
      }
    }
  }

  /// A labelled line's entries, each pending its requests (ruling 1; ruling 2 for a symbol): an e-mail entry one
  /// contact per address, any other one affiliation or legend. The entry an unmarked line under it continues.
  fn labelled_entries(
    &mut self,
    index: usize,
    entries: Vec<Entry<'_>>,
    extra: &str,
  ) -> Option<Continued> {
    let mut continues = None;
    for entry in entries {
      // (the closing half of a bracketed mark is no text of the entry: `($^1$) Univ A`, 1004.4809)
      let text = events_text(&entry.events)
        .trim_start_matches(|c: char| ")]-.:".contains(c) || c.is_whitespace())
        .to_string();
      let span = entry.events.first().and_then(|e| e.span);
      let rule = if entry.symbol {
        s!("ruling2:symbol-line{extra}")
      } else {
        s!("ruling1:request{extra}")
      };
      let contacts: Vec<&Event> = entry
        .events
        .iter()
        .copied()
        .filter(|e| e.kind == EventKind::Contact)
        .collect();
      let role = if !addresses_in(&text).is_empty()
        && contacts
          .iter()
          .all(|c| c.role.as_deref() != Some("affiliation"))
      {
        "email"
      } else if is_legend_text(&text) && entry.symbol {
        "legend"
      } else {
        "affiliation"
      };
      if role == "email" {
        for address in addresses_in(&text) {
          self
            .pending
            .push((entry.labels.clone(), entry.symbol, Assoc {
              role:   "email".into(),
              text:   address,
              label:  entry.labels.first().cloned(),
              rule:   rule.clone(),
              source: Some((index, span)),
            }));
        }
        continues = None;
      } else {
        self
          .pending
          .push((entry.labels.clone(), entry.symbol, Assoc {
            role: role.into(),
            text,
            label: entry.labels.first().cloned(),
            rule: rule.clone(),
            source: Some((index, span)),
          }));
        continues = Some(Continued::Pending(self.pending.len() - 1));
      }
    }
    continues
  }

  /// An unmarked line continuing the entry before it: its text appended.
  fn continue_entry(&mut self, carried: &Continued, text: &str) {
    let extend = |assoc: &mut Assoc| {
      assoc.text = s!("{} {text}", assoc.text);
      if !assoc.rule.contains("line:continued") {
        assoc.rule.push_str(" line:continued");
      }
    };
    match carried {
      Continued::Pending(at) => {
        if let Some((_, _, assoc)) = self.pending.get_mut(*at) {
          extend(assoc);
        }
      },
      Continued::Joined(places) => {
        for &(c, i) in places {
          if let Some(assoc) = self.out.creators[c].affiliations.get_mut(i) {
            extend(assoc);
          }
        }
      },
    }
  }

  /// A names line: names split at its separators, marks attached by adjacency, notes and contacts to the name before.
  fn names_line(
    &mut self,
    index: usize,
    policy: &'static Policy,
    cautious: bool,
    line: &Line<'_>,
    block_requests: &[String],
    group: usize,
  ) -> NamesLine {
    // (the addresses a names line prints as text are no names: `\name Shengjian Chen \email {chensj@…, chshengj@…}`,
    // jmlr2e's `\email` a font switch, 2507.03885 — each an e-mail of the line's names by the owner chain, placed once
    // the line's names are read; the words before an address in its run stay the name)
    let (owned, addresses) = names_without_addresses(&line.events);
    // (a per-author class's names line that does not read as two or more names is one name as written: aastex's
    // `\author{Sanhueza, Patricio}`, 2604.19097 — `split_author_line_cautious`, `name_count` on the line's text)
    let owned = if cautious {
      joined_single_name(owned)
    } else {
      owned
    };
    let line = &Line {
      column:      line.column,
      events:      owned.iter().collect(),
      after_blank: line.after_blank,
    };
    let made = self.names_line_events(index, policy, line, block_requests, group);
    if !addresses.is_empty() {
      let source = (index, line.events.first().and_then(|e| e.span));
      self.place_line_contacts(&addresses.join(", "), "email", &made.creators, source);
    }
    made
  }

  /// [`Resolver::names_line`] over a line's events once its addresses are out.
  fn names_line_events(
    &mut self,
    index: usize,
    policy: &'static Policy,
    line: &Line<'_>,
    block_requests: &[String],
    group: usize,
  ) -> NamesLine {
    let mut made: Vec<usize> = Vec::new();
    // the separator before the next name, and whether a mark after a comma waits for the next name
    let mut last_sep: Option<&Event> = None;
    let mut waiting: Vec<String> = Vec::new();
    let mut waiting_notes: Vec<&Event> = Vec::new();
    // the mark before, where it ends in the source and whether it went to the next name
    let mut last_mark: Option<(u16, bool)> = None;
    let split_rule = |sep: Option<&Event>| match sep.map(|e| &e.kind) {
      Some(EventKind::Sep(SepKind::Comma)) => "line:names sep:comma",
      Some(EventKind::Sep(SepKind::Word(_))) => "line:names sep:word",
      Some(EventKind::Sep(SepKind::Gap { .. })) => "line:names sep:gap",
      Some(EventKind::Sep(SepKind::And(_))) => "line:names sep:and",
      _ => "line:names",
    };
    let events = &line.events;
    for (i, event) in events.iter().enumerate() {
      match &event.kind {
        EventKind::Name if !event.text.chars().any(char::is_alphanumeric) => {},
        // (a mark printed as text after a name is its request, no name: `A.P.~Orlov$^1$, 2,` — the "2" a mark the
        // source typed outside the math, 1912.02075)
        EventKind::Name
          if !made.is_empty()
            && event.text.chars().any(|c| c.is_ascii_digit())
            && event
              .text
              .chars()
              .all(|c| c.is_ascii_digit() || ", ".contains(c))
            && event.text.len() <= 6 =>
        {
          let c = *made.last().expect("a name");
          self.out.creators[c]
            .requests
            .extend(mark_labels(&event.text));
          last_sep = None;
        },
        // (a lone placement letter beside a box is no name: `shown_arguments` reads every argument as shown when a
        // definition's parameter and argument counts differ, util_text.rs:374 — review revfms2a1_3; flagged by its rule)
        EventKind::Name if is_placement_letter(events, i) => {
          if let Some(&c) = made.last() {
            let creator = &mut self.out.creators[c];
            if !creator.rule.contains("flag:placement-letter") {
              creator.rule.push_str(" flag:placement-letter");
            }
          }
        },
        // (a name run a layout container alone parts is one name: revtex's `\firstname{Francesco}
        // \surname{Parisen Toldin}`, 1303.6104; a font switch or a box inside a name never separates, decision 6d)
        // (a name of initials alone does not end at a gap: `V.G.~ Kagramanova`, 0802.4349, a tie and a space)
        // (nor does a given name alone, or a name ending in an initial, with nothing but the gap after it: `Takumi~
        // Saegusa`, 1406.5580; `Nazlı\ \ Demirer`, 1612.07367; `James B.~ Martin`, 1806.07838 — a full name is two
        // words or more; the pieces were the wide-space XML-wrong class's by mistake)
        EventKind::Name
          if !made.is_empty()
            && last_sep.is_some_and(|s| matches!(s.kind, EventKind::Sep(SepKind::Gap { .. })))
            && (initials_only(&self.out.creators[*made.last().expect("a name")].name)
              || (only_gap_between(events, i)
                && name_continues_at_gap(
                  &self.out.creators[*made.last().expect("a name")].name,
                ))) =>
        {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          let tag = if initials_only(&creator.name) {
            "name:initials-joined"
          } else {
            "name:gap-joined"
          };
          creator.name = s!("{} {}", creator.name, event.text);
          if !creator.rule.contains(tag) {
            creator.rule.push(' ');
            creator.rule.push_str(tag);
          }
          last_sep = None;
        },
        EventKind::Name if joins_previous(events, i) && !made.is_empty() => {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          creator.name = s!("{} {}", creator.name, event.text);
          if !creator.rule.contains("name:layout-joined") {
            creator.rule.push_str(" name:layout-joined");
          }
        },
        // (a qualifier after a comma is the name's: `Robert W. Heath, Jr.`, `Matthew Nokleby, Student Member, IEEE`,
        // printed after the name, comma-separated — name_text.rs `is_name_qualifier`; 1106.5253, 1203.0695)
        EventKind::Name if !made.is_empty() && is_name_qualifier(&event.text) => {
          let c = *made.last().expect("a name");
          let creator = &mut self.out.creators[c];
          let glue = if last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma)) {
            ", "
          } else {
            " "
          };
          creator.name = s!("{}{glue}{}", creator.name, event.text);
          if !creator.rule.contains("name:qualifier") {
            creator.rule.push_str(" name:qualifier");
          }
          last_sep = None;
        },
        // (an institution in a row of names is the affiliation of the name before it: `Chen Jia, SI-TECH Information
        // Technology`, 2406.02959; the classify helpers, `reads_as_affiliation`)
        EventKind::Name
          if !made.is_empty()
            && reads_as_affiliation(&text_tokens(&event.text))
            && !is_collaboration(&event.text) =>
        {
          let c = *made.last().expect("a name");
          self.attach(
            Some(c),
            Assoc {
              role:   "affiliation".into(),
              text:   event.text.clone(),
              label:  None,
              rule:   "line:names classify:institution".into(),
              source: Some((index, event.span)),
            },
            false,
          );
          last_sep = None;
        },
        EventKind::Name => {
          let requests = [std::mem::take(&mut waiting), block_requests.to_vec()].concat();
          let c = self.new_creator(
            event.text.clone(),
            split_rule(last_sep),
            index,
            requests,
            group,
          );
          made.push(c);
          for note in std::mem::take(&mut waiting_notes) {
            self.note(index, note, Some(c), "note:adjacent");
          }
          last_sep = None;
        },
        EventKind::Sep(_) => last_sep = Some(event),
        EventKind::Mark => {
          let labels = mark_labels(event.label.as_deref().unwrap_or(""));
          let next_is_name = events[i + 1..]
            .iter()
            .find(|e| !matches!(e.kind, EventKind::Layout | EventKind::Mark))
            .is_some_and(|e| e.kind == EventKind::Name);
          let after_comma = last_sep.is_some_and(|s| s.kind == EventKind::Sep(SepKind::Comma));
          // (a mark between a separator and the next name stands with the one it is nearer to in the source: `Ann,$^{1}$
          // Bob` the name before, as revtex writes it, 0705.3409; `Ann$^1$, $^{2}$Bob` the next, a prefix — the spans'
          // distances, the math shift counted)
          let next_start = events[i + 1..]
            .iter()
            .find(|e| e.kind == EventKind::Name)
            .and_then(|e| e.span)
            .map(|s| s.0);
          let nearer_next = match (last_sep.and_then(|s| s.span), event.span, next_start) {
            (Some((_, sep_end)), Some((start, end)), Some(next)) => {
              next.saturating_sub(end) < start.saturating_sub(sep_end)
            },
            _ => false,
          };
          let inst = event.role.as_deref().is_some_and(|r| r.starts_with("inst"));
          // (a mark glued to the mark before it goes where that one went: `Oh,$^{1}$$^{,2}$ Nordlander`, mn2e,
          // 2306.15492 — the second no nearer the next name than the first; a math mark's span is inside its shifts,
          // so at most the two shifts `$$` lie between)
          let glued = match (last_mark, event.span) {
            (Some((end, to_next)), Some((start, _))) if start >= end && start <= end + 2 => {
              Some(to_next)
            },
            _ => None,
          };
          let to_next = glued.unwrap_or_else(|| {
            made.is_empty()
              || (last_sep.is_some()
                && next_is_name
                && nearer_next
                && !(inst && policy.inst_after_comma))
          });
          last_mark = event.span.map(|(_, end)| (end, to_next));
          if to_next {
            waiting.extend(labels);
          } else if let Some(&c) = made.last() {
            self.out.creators[c].requests.extend(labels);
            if after_comma {
              let rule = if inst && policy.inst_after_comma {
                "mark:inst-after-comma"
              } else {
                "mark:after-comma"
              };
              let creator = &mut self.out.creators[c];
              if !creator.rule.contains(rule) {
                creator.rule.push(' ');
                creator.rule.push_str(rule);
              }
            }
          }
        },
        EventKind::Note => match made.last() {
          Some(&c) => self.note(index, event, Some(c), "note:adjacent"),
          None => waiting_notes.push(event),
        },
        EventKind::Contact => {
          let owner: Vec<usize> = made.last().copied().into_iter().collect();
          self.contact_event(index, event, &owner, "owner:adjacent");
        },
        _ => {},
      }
    }
    for note in waiting_notes {
      self.note(index, note, made.last().copied(), "note:adjacent");
    }
    if !waiting.is_empty()
      && let Some(&c) = made.last()
    {
      self.out.creators[c].requests.extend(waiting);
    }
    NamesLine { creators: made }
  }

  /// A Contact event (an `\email`, `\url`, an in-block `\affil`): a labelled one waits for its requests; an e-mail by
  /// the owner chain among `owners`; any other to `owners` (the name before, the row above).
  fn contact_event(&mut self, index: usize, contact: &Event, owners: &[usize], rule: &str) {
    let (role, text) = contact_parts(contact);
    let assoc = Assoc {
      role: role.clone(),
      text,
      label: contact.label.clone(),
      rule: rule.to_string(),
      source: Some((index, contact.span)),
    };
    if let Some(label) = contact.label.as_deref().filter(|_| role == "affiliation") {
      self.pending.push((mark_labels(label), false, Assoc {
        rule: "ruling1:request".into(),
        ..assoc
      }));
      return;
    }
    if owners.is_empty() {
      self.attach(
        None,
        Assoc {
          rule: "owner:block".into(),
          ..assoc
        },
        false,
      );
      return;
    }
    // (an e-mail given to a row of names, or printing several addresses, is placed address by address by the owner
    // chain, ruling 3 — `\email{sfranco@kitp.ucsb.edu, ahanany@imperial.ac.uk}` under four names, 0809.3237)
    if role == "email" && (owners.len() > 1 || addresses_in(&assoc.text).len() > 1) {
      self.place_line_contacts(&assoc.text, "email", owners, (index, contact.span));
      return;
    }
    for &owner in owners {
      self.attach(Some(owner), assoc.clone(), false);
    }
  }

  /// A Note event: its inner contacts to its owner; the note itself, its whole text, unless they emptied it (ruling 4).
  fn note(&mut self, index: usize, note: &Event, owner: Option<usize>, rule: &str) {
    // (what is left of its text once its contacts are taken out decides only whether they emptied it: the note keeps
    // its whole text, "Codebase is available at https://…" — review revfms2a2 finding 1; 2401.17399, 1607.05792,
    // 2603.01292)
    let mut rest = note.text.clone();
    let contacts: Vec<&Event> = note
      .inner
      .iter()
      .filter(|e| e.kind == EventKind::Contact)
      .collect();
    for inner in &contacts {
      if !inner.text.is_empty() {
        rest = rest.replacen(inner.text.as_str(), "", 1);
      }
      let (role, text) = contact_parts(inner);
      let assoc = Assoc {
        role,
        text,
        label: inner.label.clone(),
        rule: s!("{rule} ruling4:inner-contact"),
        source: Some((index, inner.span)),
      };
      // (an e-mail goes to the author it spells before the one the note is on — charter §4, by evidence: `Chauvet\thanks{…
      // jocelyn.chauvet@…; xavier.bry@…}`, 1908.04020; 2304.00435 — read once every name is, `answer_note_emails`)
      if assoc.role == "email" {
        self.note_emails.push((owner, assoc));
      } else {
        self.attach(owner, assoc, false);
      }
    }
    let emptied = !contacts.is_empty() && rest.chars().all(|c| !c.is_alphanumeric());
    if emptied {
      self.out.dropped.push(Assoc {
        role:   "note".into(),
        text:   note.text.trim().to_string(),
        label:  note.label.clone(),
        rule:   s!("{rule} ruling4:emptied"),
        source: Some((index, note.span)),
      });
      return;
    }
    // (a note opening with a Mark some name requests is that label's legend: `\thanks{$^{1}$ Canizaro … Center}`
    // under `Newaz$^{1,2,*}$`, 2311.16145 — the mark it prints, its leading Mark event; its owners by ruling 1)
    if let Some((labels, legend)) = leading_mark(note)
      && labels
        .iter()
        .any(|l| !label_owners(&self.out.creators, l).is_empty())
    {
      let symbol = labels.iter().all(|l| is_symbol_label(l));
      let role = if is_legend_text(&legend) || symbol {
        "legend"
      } else {
        "affiliation"
      };
      self.pending.push((labels.clone(), symbol, Assoc {
        role:   role.into(),
        text:   legend.clone(),
        label:  labels.first().cloned(),
        rule:   s!("{rule} note:legend ruling1:request"),
        source: Some((index, note.span)),
      }));
      return;
    }
    let role = note
      .role
      .as_deref()
      .unwrap_or("thanks")
      .split(' ')
      .next()
      .unwrap_or("thanks")
      .to_string();
    let rule = if note.text.is_empty() && note.inner.is_empty() {
      s!("{rule} ruling4:empty-kept")
    } else {
      rule.to_string()
    };
    self.attach(
      owner,
      Assoc {
        role,
        text: note.text.trim().to_string(),
        label: note.label.clone(),
        rule,
        source: Some((index, note.span)),
      },
      true,
    );
  }

  /// An API call made apart from the blocks.
  fn api(
    &mut self,
    index: usize,
    tag: &str,
    role: &str,
    attrs: &str,
    events: &[Event],
    policy: &'static Policy,
  ) {
    let split = self.splitting;
    // its leading marks (`\affil{$^1$Univ X}`), its `\label`s, its text
    let lead: Vec<String> = events
      .iter()
      // (a space or a break before the mark does not part it from the content: `\affiliation{ $^a$Fermilab, $^b$UC
      // Davis}`, 0902.0006)
      // nor punctuation, an acmart address part's brace (`{$^1$Gaoling School …`, 2211.15148)
      .skip_while(|e| {
        matches!(
          e.kind,
          EventKind::Sep(SepKind::Gap { .. }) | EventKind::Break | EventKind::Layout
        ) || (e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
      })
      .take_while(|e| matches!(e.kind, EventKind::Mark | EventKind::Layout))
      .filter(|e| e.kind == EventKind::Mark)
      .flat_map(|e| mark_labels(e.label.as_deref().unwrap_or("")))
      .collect();
    let keys: Vec<String> = events
      .iter()
      .filter(|e| e.kind == EventKind::Layout && e.role.as_deref() == Some("label"))
      .filter_map(|e| e.label.clone())
      .map(|key| s!("ref:{key}"))
      .collect();
    // (its Contact events, an `\email` inside an `\institute` entry, are contacts of their own, with the entry's owners;
    // an address it prints as text stays in its text, as today's: `strip_addresses` split them off and disagreed with
    // 66 + 52 papers of the 5,000)
    // (an entry of another role — an `\email`, a `\thanks` contact — is its declared role whatever it holds: a
    // `mailto:` link inside an `\email` is the e-mail, review revfms2a2 finding 10, 2506.17875)
    let splits_contacts =
      tag != "ltx:creator" && matches!(role, "affiliation" | "address" | "altaffiliation");
    // (only its e-mails and ORCIDs: a link inside an affiliation is its text, as today's — `\href{…}{Institut Langevin},
    // \href{…}{ESPCI Paris}`, 2609.39586; `Stardust Labs, Ness Ziona, Israel; \url{https://www.stardust-initiative.com}`,
    // 2605.14947; 2308.00495, 2111.07288)
    // (the contacts its commands declare, `\email`, `\orcid`: a link is text there, an e-mail address or an ORCID
    // record it points to too — amsart's `\address{… \\ \href{mailto:…}{…}}`, 2408.13873, 2305.04702)
    let split_out = |e: &Event| {
      splits_contacts
        && e.kind == EventKind::Contact
        && matches!(e.role.as_deref(), Some("email" | "orcid"))
    };
    let refs: Vec<&Event> = events.iter().filter(|e| !split_out(e)).collect();
    let text = events_text(&refs);
    let role_owned = contact_role(Some(role), &text);
    let role = if tag == "ltx:contact" {
      role_owned.as_str()
    } else {
      role
    };
    let inner: Vec<Assoc> = events
      .iter()
      .filter(|e| split_out(e) && !contact_parts(e).1.is_empty())
      .map(|e| Assoc {
        role:   contact_parts(e).0,
        text:   contact_parts(e).1,
        label:  None,
        rule:   String::new(),
        source: Some((index, e.span)),
      })
      .collect();
    if tag == "ltx:creator" {
      let requests: Vec<String> = attr_value(attrs, "annotations")
        .map(|l| {
          l.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
        })
        .unwrap_or_default();
      if !text.is_empty() {
        let start = self.out.creators.len();
        self.new_creator(text, "api:creator", index, requests, index);
        if policy.thenote {
          self.thenote_author(
            start,
            attr_value(attrs, "annotations").is_some_and(|l| !l.is_empty()),
          );
        }
      }
      return;
    }
    let note = tag == "ltx:note";
    let role = if role.is_empty() { "contact" } else { role };
    // (authblk: every `\affil` makes the next `\author` step the counter; a label shows the marks)
    let thenote_affil = policy.thenote && role == "affiliation";
    if thenote_affil {
      self.thenote.newaffil = true;
      if attr_value(attrs, "label").is_some_and(|l| !l.is_empty()) {
        self.thenote.labelled = true;
      }
    }
    // (a contact printing nothing is no contact: `\IEEEauthorblockA{\\}`)
    if text.is_empty() && inner.is_empty() && !note {
      return;
    }
    // a class's `\institute{A \and B}` list numbered by its `\and`s (`labelseq=`: llncs.cls:870-885, OD #791): each
    // piece an entry of its own, numbered in turn
    if attr_value(attrs, "labelseq").is_some_and(|s| !s.is_empty())
      && policy.label_sequence
      && events
        .iter()
        .any(|e| matches!(e.kind, EventKind::Sep(SepKind::And(_))))
    {
      for piece in events.split(|e| matches!(e.kind, EventKind::Sep(SepKind::And(_)))) {
        if piece
          .iter()
          .any(|e| e.kind == EventKind::Name || e.kind == EventKind::Contact)
        {
          self.api(index, tag, role, attrs, piece, policy);
        }
      }
      return;
    }
    // an entry of several marked entries (`\affiliation{$^{*}$Univ A,\\$^{+}$Univ B}`): each entry by its own leading
    // mark, as ruling 1 reads the marks (cond-mat0510562, 1612.09425, 1809.03738, hep-ph0408045, 2509.08362,
    // 1506.00848: the whole went to the first mark's carriers)
    if !split
      && tag != "ltx:creator"
      && attr_value(attrs, "label").is_none_or(str::is_empty)
      && attr_value(attrs, "labelseq").is_none_or(|s| s.is_empty() || !policy.label_sequence)
    {
      let pieces = mark_led_pieces(events);
      if !pieces.is_empty() {
        self.splitting = true;
        for piece in pieces {
          self.api(index, tag, role, attrs, &piece, policy);
        }
        self.splitting = false;
        return;
      }
    }
    // (a class whose `\and` prints the word: the entry's text holds it, `Policy::and_word`)
    if policy.and_word
      && events
        .iter()
        .any(|e| matches!(e.kind, EventKind::Sep(SepKind::And(_))))
    {
      let worded: Vec<Event> = events
        .iter()
        .map(|e| match e.kind {
          EventKind::Sep(SepKind::And(_)) => Event {
            kind: EventKind::Sep(SepKind::Word("and".into())),
            ..e.clone()
          },
          _ => e.clone(),
        })
        .collect();
      self.api(index, tag, role, attrs, &worded, policy);
      return;
    }
    // a contact of a class's address list printed apart from the names (`owner=sequence`, recorded `_sequence=<n>`:
    // jheppub's `\emailAdd`, iopart's `\ead`; OD #805), placed once the whole list is read
    if role == "email" && attr_value(attrs, "_sequence").is_some() {
      let n = self.out.creators.len();
      let group: Vec<usize> = match n.checked_sub(1).map(|last| self.group[last]) {
        Some(last) if policy.call_is_group => (0..n).filter(|&c| self.group[c] == last).collect(),
        _ => (0..n).collect(),
      };
      let assoc = Assoc {
        role:   "email".into(),
        text:   text.clone(),
        label:  None,
        rule:   String::new(),
        source: Some((index, None)),
      };
      self.sequence_calls += 1;
      let place = attr_value(attrs, "_sequence")
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(self.sequence_calls);
      for address in addresses_in(&text) {
        self
          .sequenced
          .push((address, assoc.clone(), group.clone(), place));
      }
      return;
    }
    let mut labels: Vec<String> = Vec::new();
    let mut label_rule = "";
    if let Some(label) = attr_value(attrs, "label").filter(|l| !l.is_empty()) {
      labels = mark_labels(label);
      label_rule = "api:label";
    } else if let Some(seq) =
      attr_value(attrs, "labelseq").filter(|s| !s.is_empty() && policy.label_sequence)
    {
      let at = match self.sequence.iter_mut().find(|(r, _)| r == seq) {
        Some((_, n)) => {
          *n += 1;
          *n
        },
        None => {
          self.sequence.push((seq.to_string(), 1));
          1
        },
      };
      labels = vec![at.to_string()];
      labels.extend(keys.iter().cloned());
      label_rule = "api:labelseq";
    } else if !lead.is_empty() {
      labels = lead;
      label_rule = if split {
        "api:leading-mark api:split-by-marks"
      } else {
        "api:leading-mark"
      };
    } else if !keys.is_empty() {
      labels = keys.clone();
      label_rule = "api:ref-label";
    }
    let assoc = Assoc {
      role:   role.to_string(),
      text:   text.clone(),
      label:  labels.first().cloned(),
      rule:   String::new(),
      source: Some((index, None)),
    };
    if !labels.is_empty() {
      for contact in inner {
        self.pending.push((labels.clone(), false, Assoc {
          rule: s!("{label_rule} api:inner-contact"),
          label: labels.first().cloned(),
          ..contact
        }));
      }
      if !text.is_empty() {
        self.pending.push((labels, false, Assoc {
          rule: label_rule.to_string(),
          ..assoc
        }));
      }
      // (today's call attaches the entry to its `annotate=new` run first and moves it by its label after: the run's
      // authors hold the role, so a later unlabelled entry's run stops at them — revtex's `\affiliation{$^1$Univ A}`
      // under Tiemann, then `\affiliation{Current address: TRUMPF}` under Kahmann alone, 1506.04004)
      // (a mark the entry prints only: an entry labelled by its call, `\address[l1]`, waits for its label alone, as
      // today's — elsarticle, 1712.00945)
      let annotate = attr_value(attrs, "annotate").unwrap_or("");
      if label_rule.starts_with("api:leading-mark")
        && (annotate == "new" || annotate.starts_with("run"))
      {
        for c in self.new_run(role) {
          self.roles[c].push(role.to_string());
        }
      }
      return;
    }
    if text.is_empty() && inner.is_empty() {
      return;
    }
    let annotate = attr_value(attrs, "annotate").unwrap_or("");
    let n = self.out.creators.len();
    let thenote_rule: String;
    // (a note made before any author is the title's: revtex's `\title{…\thanks{…}}`, 1603.01892, 1802.05590,
    // cond-mat0502298 — no creator's, whatever the call's parent; the block's, as today's orphan note, OD #159)
    if n == 0 && (note || role == "thanks") {
      self.attach(
        None,
        Assoc {
          rule: "api:owner:block api:title-note".into(),
          ..assoc.clone()
        },
        note,
      );
      return;
    }
    let (owners, rule): (Vec<usize>, &str) = if n == 0 && !annotate.is_empty() {
      // (an annotation of the new authors, or of a number of them, before any creator reaches none, as today's
      // `\lx@annotate@frontmatter@now` counts its parents: revtex collaboration lists declaring every `\affiliation`
      // before the first `\author` to fix their numbering, 2103.04797, 0704.1604; an unqualified one is the block's,
      // 0709.0811)
      self.out.dropped.push(Assoc {
        rule: "api:no-creator".into(),
        ..assoc.clone()
      });
      return;
    } else if n == 0 {
      (Vec::new(), "api:owner:block")
    } else if annotate == "all" {
      ((0..n).collect(), "api:annotate:all")
    } else if annotate == "new" || annotate.starts_with("run") {
      // (`new`: back from the last author to one holding the role, Perl Base_Utility.pool.ltxml:520-536; `run[:<cap>]`:
      // the same, and a second such contact before any new author the same run's, at most <cap> authors —
      // frontmatter_api.rs `annotate=run`, authblk's `\AB@thenote` group)
      let run_rule = annotate.starts_with("run");
      let cap = annotate
        .split_once(':')
        .and_then(|(_, c)| c.trim().parse::<usize>().ok())
        .filter(|c| *c >= 1);
      let run = match &self.last_run {
        Some((r, count, run)) if run_rule && r == role && *count == n => run.clone(),
        _ => {
          let mut run = Vec::new();
          for c in (0..n).rev() {
            if !run.is_empty() && self.roles[c].iter().any(|r| r == role) {
              break;
            }
            if cap.is_some_and(|cap| run.len() >= cap) {
              break;
            }
            run.push(c);
          }
          run.reverse();
          run
        },
      };
      if run_rule {
        self.last_run = Some((role.to_string(), n, run.clone()));
      }
      if run_rule && thenote_affil {
        // (its `\AB@thenote`: when the marks show it is the label, `thenote_finish`)
        let tag = s!("thenote:{}", self.thenote.affils.len());
        self
          .thenote
          .affils
          .push((self.thenote.counter, tag.clone()));
        thenote_rule = s!("api:annotate:run {tag}");
        (run, thenote_rule.as_str())
      } else {
        (
          run,
          if run_rule {
            "api:annotate:run"
          } else {
            "api:annotate:new"
          },
        )
      }
    } else if let Ok(k) = annotate.parse::<usize>() {
      ((n.saturating_sub(k)..n).collect(), "api:annotate:n")
    } else if role == "email" && policy.email_owner == EmailOwner::SpelledElseOrder {
      // (the class's list printed apart from the names: placed once the whole list is read, `answer_sequence`)
      let group: Vec<usize> = if policy.call_is_group {
        let last = self.group[n - 1];
        (0..n).filter(|&c| self.group[c] == last).collect()
      } else {
        (0..n).collect()
      };
      self.sequence_calls += 1;
      for address in addresses_in(&text) {
        self
          .sequenced
          .push((address, assoc.clone(), group.clone(), self.sequence_calls));
      }
      return;
    } else {
      (vec![n - 1], "api:annotate:last")
    };
    let owners: Vec<Option<usize>> = if owners.is_empty() {
      vec![None]
    } else {
      owners.into_iter().map(Some).collect()
    };
    for owner in owners {
      if !text.is_empty() || note {
        self.attach(
          owner,
          Assoc {
            rule: rule.to_string(),
            ..assoc.clone()
          },
          note,
        );
      }
      for contact in &inner {
        self.attach(
          owner,
          Assoc {
            rule: s!("{rule} api:inner-contact"),
            ..contact.clone()
          },
          false,
        );
      }
    }
  }

  /// A sole author's orphans are that author's, as Perl attaches them (OXIDIZED_DESIGN_DIVERGENCES #159, relocate.rs):
  /// amsart's `\address` or `\dedicatory` written before the `\author` (0911.5235, 1102.3253, 0712.4133); its
  /// affiliations only when it holds none (an author whose marks request one is not given the one no one marks). A note
  /// of the title (`api:title-note`) stays the block's.
  fn sole_author_takes_the_block(&mut self) {
    if self.out.creators.len() != 1 || self.out.block.is_empty() {
      return;
    }
    let affiliated = !self.out.creators[0].affiliations.is_empty();
    let mut kept = Vec::new();
    for assoc in std::mem::take(&mut self.out.block) {
      let institute = matches!(assoc.role.as_str(), "affiliation" | "address");
      let note = matches!(
        assoc.role.as_str(),
        "thanks" | "legend" | "note" | "footnotetext"
      );
      if assoc.rule.contains("api:title-note") || (institute && affiliated) {
        kept.push(assoc);
        continue;
      }
      let rule = s!("{} owner:sole-author", assoc.rule);
      self.attach(Some(0), Assoc { rule, ..assoc }, note);
    }
    self.out.block = kept;
  }

  /// The run an `annotate=new` entry of `role` reaches: back from the last author to one holding the role (Perl
  /// Base_Utility.pool.ltxml:520-536).
  fn new_run(&self, role: &str) -> Vec<usize> {
    let mut run = Vec::new();
    for c in (0..self.out.creators.len()).rev() {
      if !run.is_empty() && self.roles[c].iter().any(|r| r == role) {
        break;
      }
      run.push(c);
    }
    run.reverse();
    run
  }

  /// A class's address list (OD #805): each address to the author it spells; else, when the list has one address per
  /// author and every spelled one is at its author's place, to the author at its place; else the group's only author,
  /// the one corresponding author, the block.
  fn answer_sequence(&mut self) {
    let list = std::mem::take(&mut self.sequenced);
    let all: Vec<usize> = (0..self.out.creators.len()).collect();
    let spelled: Vec<Option<usize>> = list.iter().map(|(a, ..)| self.spelled(a, &all)).collect();
    // (one place per author, the places the calls': an author's two addresses in one call share it, OD #805)
    let mut places: Vec<usize> = list.iter().map(|(_, _, _, place)| *place).collect();
    places.sort_unstable();
    places.dedup();
    let place_of = |place: usize| places.iter().position(|p| *p == place);
    let in_order = places.len() == all.len()
      && list
        .iter()
        .zip(&spelled)
        .all(|((_, _, _, place), owner)| owner.is_none_or(|o| place_of(*place) == Some(o)));
    for ((address, assoc, group, place), spelled) in list.into_iter().zip(spelled) {
      let (owner, rule) = match spelled {
        Some(owner) => (Some(owner), "owner:spelled"),
        None if in_order => (place_of(place), "owner:order"),
        None => match group.as_slice() {
          [only] => (Some(*only), "owner:only-author"),
          _ => self.corresponding(&all),
        },
      };
      self.attach(
        owner,
        Assoc {
          role: "email".into(),
          text: address,
          rule: s!("api:sequence {rule}"),
          ..assoc
        },
        false,
      );
    }
  }

  /// An `\author` call under authblk's notes, its creators from `start` on: the counter stepped at the first after an
  /// `\affil`; each unlabelled creator showing it, a labelled one showing the marks.
  fn thenote_author(&mut self, start: usize, labelled: bool) {
    if self.thenote.newaffil {
      self.thenote.counter += 1;
      self.thenote.newaffil = false;
    }
    if labelled {
      self.thenote.labelled = true;
      return;
    }
    for c in start..self.out.creators.len() {
      self.thenote.run_of.push((c, self.thenote.counter));
    }
  }

  /// authblk's notes once every call is read: when the marks show (a label was given, or the counter passed
  /// `\c@Maxaffil`, 2 unless the document set it), each unlabelled author requests its number and each unlabelled
  /// `\affil` is labelled by its number (ruling 1: `\author[1]{Ann}\author{Bob}\affil[1]{X}\affil{Y}` prints Y under
  /// "Ann¹ Bob¹" both, `\author[a]{Ann}` under Bob alone — 2605.25017); else each stays on its run.
  fn thenote_finish(&mut self) {
    let note = std::mem::take(&mut self.thenote);
    if note.affils.is_empty() && note.run_of.is_empty() {
      return;
    }
    let shown = note.labelled || note.counter as i64 > note.maxaffil.unwrap_or(2);
    if !shown {
      return;
    }
    for (c, number) in note.run_of {
      self.out.creators[c].requests.push(number.to_string());
      self.thenote_requests.push((c, number.to_string()));
    }
    for (number, tag) in note.affils {
      let mut moved: Option<Assoc> = None;
      for c in 0..self.out.creators.len() {
        let creator = &mut self.out.creators[c];
        let before = creator.affiliations.len();
        let mut kept = Vec::new();
        let mut removed: Vec<usize> = Vec::new();
        for (i, a) in std::mem::take(&mut creator.affiliations)
          .into_iter()
          .enumerate()
        {
          if a.rule.split_whitespace().any(|t| t == tag) {
            removed.push(i);
            moved.get_or_insert(a);
          } else {
            kept.push(a);
          }
        }
        creator.affiliations = kept;
        if removed.is_empty() || before == 0 {
          continue;
        }
        // (the attach order without them, the places after each shifted down)
        self.order[c] = self.order[c]
          .iter()
          .filter(|(aff, i)| !(*aff && removed.contains(i)))
          .map(|&(aff, i)| {
            (
              aff,
              if aff {
                i - removed.iter().filter(|r| **r < i).count()
              } else {
                i
              },
            )
          })
          .collect();
      }
      if let Some(assoc) = moved {
        let rule = s!("api:thenote ruling1:request");
        self.pending.push((vec![number.to_string()], false, Assoc {
          rule,
          label: Some(number.to_string()),
          ..assoc
        }));
      }
    }
  }

  /// Today's redistribution of a contact pile (insert.rs `distribute_upfront_contacts`, a surpass of Perl;
  /// html_feedback#46, 2308.06214; 2205.02163): when every contact went to the last of N ≥ 2 authors (amsart's
  /// `\author{A and B}` then `\address{…}\email{…}` twice), K = N·m of them, their roles periodic with period m, the
  /// j-th group of m is the j-th author's. Any other pile stays as it is.
  fn distribute_upfront(&mut self) {
    let n = self.out.creators.len();
    if n < 2 || self.order[..n - 1].iter().any(|o| !o.is_empty()) {
      return;
    }
    let order = self.order[n - 1].clone();
    let k = order.len();
    if k == 0 || !k.is_multiple_of(n) {
      return;
    }
    let m = k / n;
    let last = &self.out.creators[n - 1];
    let role_at = |&(aff, i): &(bool, usize)| {
      if aff {
        last.affiliations[i].role.clone()
      } else {
        last.contacts[i].role.clone()
      }
    };
    let roles: Vec<String> = order.iter().map(role_at).collect();
    if (0..k - m).any(|i| roles[i] != roles[i + m]) {
      return;
    }
    let mut moved: Vec<(usize, Assoc, bool)> = Vec::new();
    for (j, group) in order.chunks(m).enumerate().take(n - 1) {
      for &(aff, i) in group {
        let assoc = if aff {
          &self.out.creators[n - 1].affiliations[i]
        } else {
          &self.out.creators[n - 1].contacts[i]
        };
        let mut assoc = assoc.clone();
        assoc.rule.push_str(" owner:upfront-grid");
        moved.push((j, assoc, aff));
      }
    }
    let keep: Vec<(bool, usize)> = order[(n - 1) * m..].to_vec();
    let last = &mut self.out.creators[n - 1];
    let affiliations: Vec<Assoc> = keep
      .iter()
      .filter(|(aff, _)| *aff)
      .map(|&(_, i)| last.affiliations[i].clone())
      .collect();
    let contacts: Vec<Assoc> = keep
      .iter()
      .filter(|(aff, _)| !*aff)
      .map(|&(_, i)| last.contacts[i].clone())
      .collect();
    last.affiliations = affiliations;
    last.contacts = contacts;
    for (j, assoc, aff) in moved {
      if aff {
        self.out.creators[j].affiliations.push(assoc);
      } else {
        self.out.creators[j].contacts.push(assoc);
      }
    }
  }

  /// Glued labels parted by the marks the other side shows (ruling 1: the marks shown on a name are its requests, read
  /// as the reader reads them): a name's mark that no entry labels, but that is two or more of the entries' labels
  /// written together — `12` under entries `1` and `2` (2103.10685), `1⋆` under `1` and a symbol (2606.01911), `∗123`
  /// — requests each; an entry's label no name shows, but that is the names' marks written together (`∗∗` where the
  /// name shows `∗` beside revtex's own footnote `∗`, 1505.06831), is each of them. A label some entry or name has as
  /// written is never parted.
  fn part_glued_labels(&mut self) {
    let bare = |l: &str| unprefixed(l).unwrap_or(l).to_string();
    let special = |l: &str| {
      ["ref:", "fuzzy:", "author:"]
        .iter()
        .any(|p| l.starts_with(p))
    };
    let entry_labels: Vec<String> = self
      .pending
      .iter()
      .flat_map(|(labels, ..)| labels.iter().filter(|l| !special(l)).map(|l| bare(l)))
      .collect();
    if entry_labels.is_empty() {
      return;
    }
    // (a number is parted into digits only when every entry's number is a digit: `12` under entries 1-4, 2103.10685 —
    // never `21` under twenty numbered entries, 1501.05343)
    let digit_entries = entry_labels
      .iter()
      .filter(|l| l.chars().all(|c| c.is_ascii_digit()))
      .all(|l| l.len() == 1);
    for c in 0..self.out.creators.len() {
      let requests = std::mem::take(&mut self.out.creators[c].requests);
      let mut parted: Vec<String> = Vec::new();
      let mut parted_any = false;
      let mut out = Vec::with_capacity(requests.len());
      for request in requests {
        let b = bare(&request);
        let prefix = &request[..request.len() - b.len()];
        if !special(&request)
          && !entry_labels.contains(&b)
          && (digit_entries || !b.chars().all(|c| c.is_ascii_digit()))
          && let Some(parts) = parted_label(&b, &entry_labels, true)
        {
          for p in parts {
            parted.push(s!("{prefix}{p}"));
            out.push(s!("{prefix}{p}"));
          }
          parted_any = true;
        } else {
          out.push(request);
        }
      }
      self.out.creators[c].requests = out;
      if parted_any && !self.out.creators[c].rule.contains("mark:parted") {
        self.out.creators[c].rule.push_str(" mark:parted");
      }
      self.parted.extend(parted.into_iter().map(|p| (c, p)));
    }
    let shown: Vec<String> = self
      .out
      .creators
      .iter()
      .flat_map(|c| c.requests.iter().filter(|r| !special(r)).map(|r| bare(r)))
      .collect();
    for (labels, _, assoc) in &mut self.pending {
      // (only a label a printed mark gave: never a class's numbering, `labelseq` — `\institute{… \and …}`'s 12th
      // entry is no glued `1` and `2`, 1005.2409 — nor a call's own `label=`)
      let printed = assoc.rule.split_whitespace().next().is_some_and(|first| {
        matches!(
          first,
          "ruling1:request" | "ruling2:symbol-line" | "api:leading-mark"
        )
      }) || assoc.rule.contains("note:legend");
      if !printed {
        continue;
      }
      let mut parted_any = false;
      let mut out = Vec::with_capacity(labels.len());
      for label in labels.drain(..) {
        let b = bare(&label);
        if !special(&label)
          && !shown.contains(&b)
          && let Some(parts) = parted_label(&b, &shown, false)
        {
          for p in parts {
            if !out.contains(&p) {
              out.push(p);
            }
          }
          parted_any = true;
        } else {
          out.push(label);
        }
      }
      *labels = out;
      if parted_any {
        assoc.rule.push_str(" ruling1:parted");
      }
    }
  }

  /// A note's e-mails: each to the author of the note's block it spells (FIDELITY_CHARTER §4: ownership by evidence —
  /// the spelling before the note's place), else the author the note is on (its mark is shown on that name: the
  /// main-session decision of 2026-10-22 recorded in OXIDIZED_DESIGN_DIVERGENCES #837).
  fn answer_note_emails(&mut self) {
    for (owner, assoc) in std::mem::take(&mut self.note_emails) {
      let candidates: Vec<usize> = match owner {
        Some(at) => (0..self.out.creators.len())
          .filter(|&c| self.group[c] == self.group[at])
          .collect(),
        None => (0..self.out.creators.len()).collect(),
      };
      match self.spelled(&assoc.text, &candidates) {
        Some(spelled) if Some(spelled) != owner => {
          let rule = s!("{} owner:spelled", assoc.rule);
          self.attach(Some(spelled), Assoc { rule, ..assoc }, false);
        },
        _ => self.attach(owner, assoc, false),
      }
    }
  }

  /// The labelled entries, each to every creator requesting one of its labels (ruling 1; ruling 2 for a symbol), or
  /// the block's.
  fn answer_pending(&mut self) {
    self.part_glued_labels();
    for (labels, symbol, assoc) in std::mem::take(&mut self.pending) {
      let mut owners: Vec<usize> = Vec::new();
      for label in &labels {
        for owner in label_owners(&self.out.creators, label) {
          if !owners.contains(&owner) {
            owners.push(owner);
          }
        }
      }
      owners.sort_unstable();
      // (a `fuzzy:<name>` label — svjour's `\institute` entries after `\at`, Perl's fuzzy label of a creator, Tag
      // ltx:creator — names the creator whose name, without its marks, it spells: ruling 1)
      let (owners, assoc) = match labels.iter().find_map(|l| l.strip_prefix("fuzzy:")) {
        Some(name) if owners.is_empty() => {
          let letters = tex_letters(name);
          let key = fold_name(&letters);
          let fuzzy: Vec<usize> = self
            .out
            .creators
            .iter()
            .enumerate()
            .filter(|(_, c)| !key.is_empty() && fold_name(&c.name) == key)
            .map(|(i, _)| i)
            .collect();
          let (fuzzy, rule) = if fuzzy.is_empty() {
            (
              surname_owners(&self.out.creators, &letters),
              s!("{} ruling1:fuzzy-label fuzzy:surname", assoc.rule),
            )
          } else {
            (fuzzy, s!("{} ruling1:fuzzy-label", assoc.rule))
          };
          (fuzzy, Assoc { rule, ..assoc })
        },
        _ => (owners, assoc),
      };
      let note = assoc.role == "legend";
      if owners.is_empty() {
        let rule = if symbol {
          "ruling2:uncarried owner:block"
        } else {
          "label:unrequested owner:block"
        };
        self.attach(
          None,
          Assoc {
            rule: s!("{} {rule}", assoc.rule),
            ..assoc
          },
          note,
        );
        continue;
      }
      for owner in owners {
        // (an owner found by the number authblk printed on it: ruling 1 on that mark)
        let by_thenote = self
          .thenote_requests
          .iter()
          .any(|(c, r)| *c == owner && labels.contains(r));
        let mut assoc = assoc.clone();
        if by_thenote && !assoc.rule.contains("ruling1:") {
          assoc.rule.push_str(" ruling1:thenote");
        }
        // (an owner found by a request its glued mark was parted into: ruling 1 on that mark)
        let by_parted = self
          .parted
          .iter()
          .any(|(c, r)| *c == owner && labels.iter().any(|l| label_matches(r, l)));
        if by_parted && !assoc.rule.contains("ruling1:parted") {
          assoc.rule.push_str(" ruling1:parted");
        }
        self.attach(Some(owner), assoc, note);
      }
    }
  }
}

/// The creators a fuzzy label's names name by their surnames when the label spells no name whole (today's
/// OXIDIZED_DESIGN_DIVERGENCES #159 reading, relocate.rs: svjour3's `\institute{B. Wickramanayake \at …}` for "Bemali
/// Wickramanayake", 2011.14024; `Corresponding author: Guangming Zhu, Liang Zhang, and Syed Afaq Ali Shah \at …`,
/// 2106.12864; 2403.19676; `A.M.Bykov` 1205.2208): the label's names (commas, `&`, "and") each the one creator whose
/// surname (the last word) is its last word; none when a name names no one or several.
fn surname_owners(creators: &[RCreator], letters: &str) -> Vec<usize> {
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
fn parted_label(label: &str, known: &[String], symbols: bool) -> Option<Vec<String>> {
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

/// The entries of an API call's content whose first ink is a mark, when two or more marks open one — a mark at its
/// start, after a line break or after a separator (a comma, a gap, a word) with ink since the entry before (`$^{*}$Univ
/// A,\\$^{+}$Univ B`, cond-mat0510562; `$^1$SJTU, $^2$Beihang`, 1809.03738): each entry its events up to the next,
/// the separators between them dropped. A mark glued to the ink before it (`Univ A$^{1}$`) opens none. Empty when the
/// content is one entry.
fn mark_led_pieces(events: &[Event]) -> Vec<Vec<Event>> {
  let first_ink = events.iter().find(|e| {
    inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
  });
  if first_ink.is_none_or(|e| e.kind != EventKind::Mark) {
    return Vec::new();
  }
  let mut pieces: Vec<Vec<Event>> = vec![Vec::new()];
  let mut after_sep = true;
  let mut ink = false;
  for event in events {
    match &event.kind {
      EventKind::Mark if after_sep && ink => {
        pieces.push(vec![event.clone()]);
        ink = false;
        after_sep = false;
        continue;
      },
      EventKind::Mark => {},
      EventKind::Break | EventKind::Sep(_) => {
        after_sep = true;
        pieces.last_mut().expect("a piece").push(event.clone());
        continue;
      },
      EventKind::Layout => {},
      _ => ink = true,
    }
    after_sep = matches!(event.kind, EventKind::Layout) && after_sep;
    pieces.last_mut().expect("a piece").push(event.clone());
  }
  for piece in &mut pieces {
    while piece
      .last()
      .is_some_and(|e| matches!(e.kind, EventKind::Break | EventKind::Sep(_)))
    {
      piece.pop();
    }
  }
  if pieces.len() < 2 { Vec::new() } else { pieces }
}

/// A names line's events with the e-mail addresses its Name runs print taken out (the run's words before its first
/// address stay the name; a run of addresses alone goes), and those addresses.
fn names_without_addresses(events: &[&Event]) -> (Vec<Event>, Vec<String>) {
  let mut out = Vec::with_capacity(events.len());
  let mut addresses = Vec::new();
  for event in events {
    if event.kind == EventKind::Name && !addresses_in(&event.text).is_empty() {
      let words: Vec<&str> = event.text.split_whitespace().collect();
      let first = words
        .iter()
        .position(|w| !plain_addresses(w).is_empty())
        .unwrap_or(words.len());
      addresses.extend(addresses_in(&event.text));
      if first > 0 {
        out.push(Event {
          text: words[..first].join(" "),
          ..(*event).clone()
        });
      }
      continue;
    }
    out.push((*event).clone());
  }
  (out, addresses)
}

/// A per-author class's names line as one Name when its pieces, parted only by commas and words, do not read as two
/// or more names (`name_count` on the line's text: `Sanhueza, Patricio` one name, `Ann Able, Bob Baker` two) and none
/// reads as an institution (that one is the name's affiliation, `classify:institution`).
fn joined_single_name(events: Vec<Event>) -> Vec<Event> {
  let names: Vec<usize> = (0..events.len())
    .filter(|&i| {
      events[i].kind == EventKind::Name && events[i].text.chars().any(char::is_alphanumeric)
    })
    .collect();
  let (Some(&first), Some(&last)) = (names.first(), names.last()) else {
    return events;
  };
  if names.len() < 2
    || events[first..=last].iter().any(|e| {
      !matches!(
        e.kind,
        EventKind::Name | EventKind::Sep(SepKind::Comma | SepKind::Word(_))
      )
    })
    || names
      .iter()
      .any(|&i| reads_as_affiliation(&text_tokens(&events[i].text)))
  {
    return events;
  }
  let refs: Vec<&Event> = events[first..=last].iter().collect();
  let text = events_text(&refs);
  if name_count(&text) >= 2 {
    return events;
  }
  let joined = Event {
    text,
    span: match (events[first].span, events[last].span) {
      (Some((a, _)), Some((_, b))) => Some((a, b)),
      (a, b) => a.or(b),
    },
    ..events[first].clone()
  };
  let mut out = events[..first].to_vec();
  out.push(joined);
  out.extend_from_slice(&events[last + 1..]);
  out
}

/// Whether the Name event at `i` follows a Name with only a gap (and layout) between them.
fn only_gap_between(events: &[&Event], i: usize) -> bool {
  let mut before = events[..i]
    .iter()
    .rev()
    .filter(|e| e.kind != EventKind::Layout);
  matches!(
    before.next().map(|e| &e.kind),
    Some(EventKind::Sep(SepKind::Gap { .. }))
  ) && before.next().is_some_and(|e| e.kind == EventKind::Name)
}

/// Whether a name read so far is no whole name yet: one word (a given name, `Takumi`), or ending in an initial
/// (`James B.`, `Sanhan M.S.`); a qualifier is whole.
fn name_continues_at_gap(name: &str) -> bool {
  let words: Vec<&str> = name.split_whitespace().collect();
  match words.as_slice() {
    [one] => one.chars().any(char::is_alphabetic) && !is_name_qualifier(one),
    [.., last] => initials_only(last),
    [] => false,
  }
}

/// Whether a names line's piece is a collaboration listed among the authors (`and the MAST Team`, 1609.09534; `and the
/// VVDS team`, astro-ph0509500): "the" before a team, collaboration, consortium or group's name — an author, as today's.
fn is_collaboration(text: &str) -> bool {
  let t = text.trim().to_lowercase();
  let t = t.strip_prefix("and ").unwrap_or(&t).trim();
  t.starts_with("the ")
    && [
      "team",
      "collaboration",
      "consortium",
      "group",
      "collaborations",
    ]
    .iter()
    .any(|w| t.ends_with(w))
}

/// Whether a name is initials alone (`V.G.`, `A. B.`): each word a capital with at most one small letter and a dot.
fn initials_only(name: &str) -> bool {
  let words: Vec<&str> = name.split_whitespace().collect();
  !words.is_empty()
    && words.iter().all(|w| {
      w.split_inclusive('.').all(|part| {
        let letters: Vec<char> = part.trim_end_matches('.').chars().collect();
        part.ends_with('.')
          && letters.first().is_some_and(|c| c.is_uppercase())
          && letters.len() <= 2
          && letters.iter().skip(1).all(|c| c.is_lowercase())
      })
    })
}

/// Whether the Name event at `i` continues the Name before it: only layout events (a container, a font change)
/// between them.
fn joins_previous(events: &[&Event], i: usize) -> bool {
  let between: Vec<&&Event> = events[..i]
    .iter()
    .rev()
    .take_while(|e| e.kind == EventKind::Layout)
    .collect();
  !between.is_empty()
    && !between.iter().any(|e| is_image(e))
    && events[..i - between.len()]
      .last()
      .is_some_and(|e| e.kind == EventKind::Name)
}

/// Whether a Layout event is an image (a logo, an affiliation marker: 2503.08161; review revfms2a1_3): it parts the
/// runs beside it, as a separator, never joins them into one name.
fn is_image(event: &Event) -> bool {
  event.kind == EventKind::Layout
    && event
      .role
      .as_deref()
      .is_some_and(|r| r.contains("includegraphics") || r.contains("graphic"))
}

/// Whether the Name event at `i` is a lone placement letter (`t`, `b`, `c`, `l`, `r`) next to a box (a Layout event).
fn is_placement_letter(events: &[&Event], i: usize) -> bool {
  matches!(events[i].text.as_str(), "t" | "b" | "c" | "l" | "r")
    && (i
      .checked_sub(1)
      .is_some_and(|j| events[j].kind == EventKind::Layout)
      || events
        .get(i + 1)
        .is_some_and(|e| e.kind == EventKind::Layout))
}

/// The names of a run of events (an entry of a prefix-marked line): its Name events' texts.
fn names_of(events: &[&Event]) -> Vec<String> {
  events
    .iter()
    .filter(|e| e.kind == EventKind::Name)
    .map(|e| e.text.clone())
    .collect()
}

/// Resolve a document's ledger (its inputs in call order).
pub fn resolve(inputs: &[Input]) -> Resolved {
  let mut resolver = Resolver {
    out:              Resolved::default(),
    roles:            Vec::new(),
    sequence:         Vec::new(),
    pending:          Vec::new(),
    group:            Vec::new(),
    sequenced:        Vec::new(),
    sequence_calls:   0,
    last_run:         None,
    order:            Vec::new(),
    thenote:          TheNote {
      newaffil: true,
      ..TheNote::default()
    },
    thenote_requests: Vec::new(),
    epoch_of:         Vec::new(),
    replaced:         Vec::new(),
    note_emails:      Vec::new(),
    parted:           Vec::new(),
    splitting:        false,
  };
  let mut policy: &'static Policy = &policy::KERNEL;
  for input in inputs {
    match input {
      Input::Block {
        index,
        binding,
        hook,
        keyvals,
        counter,
        replace,
        epoch,
        events,
      } => {
        policy = policy_for(binding, hook);
        // (a call replacing the authors before it in its queue: `\author{A, B}` then `\author{A\thanks{…}, B}`,
        // 2502.11323, 2509.11381 — the later one's are the document's)
        if *replace {
          for (c, e) in resolver.epoch_of.iter().enumerate() {
            if *e == Some(*epoch) {
              resolver.replaced.push(c);
            }
          }
        }
        let start = resolver.out.creators.len();
        // (a per-author class binding's call, which splits its names only where they read as names: the
        // `\lx@add@authors@append` hook, `split_author_line_cautious`)
        let cautious = hook == "\\lx@add@authors@append";
        resolver.block(*index, policy, cautious, keyvals.as_deref(), events);
        resolver.epoch_of.resize(start, None);
        resolver
          .epoch_of
          .resize(resolver.out.creators.len(), Some(*epoch));
        if policy.thenote {
          if counter.is_some() {
            resolver.thenote.maxaffil = *counter;
          }
          let labelled = keyvals
            .as_deref()
            .and_then(|kv| attr_value(kv, "annotations"))
            .is_some_and(|l| !l.is_empty());
          resolver.thenote_author(start, labelled);
        }
      },
      Input::Api {
        index,
        tag,
        role,
        attrs,
        events,
      } => resolver.api(*index, tag, role, attrs, events, policy),
    }
  }
  resolver.thenote_finish();
  resolver.answer_note_emails();
  resolver.answer_pending();
  resolver.answer_sequence();
  resolver.distribute_upfront();
  resolver.sole_author_takes_the_block();
  let mut out = resolver.out;
  if !resolver.replaced.is_empty() {
    let mut at = 0usize;
    out.creators.retain(|_| {
      at += 1;
      !resolver.replaced.contains(&(at - 1))
    });
  }
  out
}

/// The resolver's inputs from the ledger: each block rehearsed (its events), and each API call made apart from a
/// block's expansion that is a creator or a creator's annotation ([`creator_call`]).
pub(in crate::base_utilities) fn ledger_inputs(ledger: &ledger::Ledger) -> Vec<Input> {
  let mut inputs = Vec::new();
  for entry in &ledger.entries {
    match entry {
      ledger::Entry::Author(block) => {
        if let ledger::Outcome::Events { events, .. } = &block.outcome {
          inputs.push(Input::Block {
            index:   block.index,
            binding: block.binding.clone(),
            hook:    block.hook.to_string(),
            keyvals: block.keyvals.clone(),
            counter: block.counter,
            replace: block.replace,
            epoch:   block.epoch,
            events:  events.clone(),
          });
        }
      },
      ledger::Entry::Api(call) if !call.inside && creator_call(&call.tag, &call.parent) => inputs
        .push(Input::Api {
          index:  call.index,
          tag:    call.tag.clone(),
          role:   call.role.clone(),
          attrs:  call.attrs.clone(),
          events: call.events.clone(),
        }),
      _ => {},
    }
  }
  inputs
}

/// Whether an API call is the resolver's: a creator of its own (`\lx@add@creator`, tag `ltx:creator`), or an
/// annotation of the creators (`\lx@annotate@frontmatter{ltx:creator}…`) — not a document's note of its own (MDPI's
/// `reftitle`, an author-contributions note: no creator holds them in the XML, review revfms2a2 finding 7, 2608.12223),
/// nor a title's.
pub fn creator_call(tag: &str, parent: &str) -> bool {
  (tag == "ltx:creator" && parent.is_empty()) || parent == "ltx:creator"
}

#[cfg(test)]
mod tests {
  use super::*;

  fn creator(name: &str, requests: &[&str]) -> RCreator {
    RCreator {
      name: name.into(),
      requests: requests.iter().map(|r| r.to_string()).collect(),
      ..RCreator::default()
    }
  }

  /// Item 7 (review revfms2a2 finding 7): only creators and creator annotations are the resolver's — not a document's
  /// note of its own (MDPI's `reftitle`, 2608.12223), nor a title's.
  #[test]
  fn intake_takes_creator_calls_only() {
    assert!(creator_call("ltx:creator", ""));
    assert!(creator_call("ltx:contact", "ltx:creator"));
    assert!(creator_call("ltx:note", "ltx:creator"));
    assert!(!creator_call("ltx:note", ""));
    assert!(!creator_call("ltx:note", "ltx:title"));
    assert!(!creator_call("ltx:contact", ""));
  }

  /// Items 3 and 4: `author:N` names the N-th creator exactly; any other label falls back past the prefixes in Perl's
  /// order, the first tier that finds one deciding.
  #[test]
  fn label_owners_exact_then_unprefixed() {
    let creators = [
      creator("Ann", &["affiliation:1"]),
      creator("Bob", &["fn:label1"]),
      creator("Cat", &["2"]),
    ];
    assert_eq!(label_owners(&creators, "author:2"), [1]);
    assert_eq!(label_owners(&creators, "author:9"), Vec::<usize>::new());
    assert_eq!(label_owners(&creators, "1"), [0]);
    assert_eq!(label_owners(&creators, "label1"), [1]);
    assert_eq!(label_owners(&creators, "affiliation:2"), [2]);
    assert_eq!(label_owners(&creators, "institute:1"), [0]);
  }

  /// Item 2: a `fuzzy:` label and a `\ref` key stay whole; the name's letters skip its marks.
  #[test]
  fn fuzzy_labels_and_their_letters() {
    assert_eq!(mark_labels("fuzzy:Luca Passalacqua (\\Envelope) "), [
      "fuzzy:Luca Passalacqua (\\Envelope)"
    ]);
    assert_eq!(mark_labels("*1, a"), ["*", "1", "a"]);
    assert_eq!(
      fold_name(&tex_letters("Luca Passalacqua (\\Envelope)")),
      "lucapassalacqua"
    );
    assert_eq!(
      fold_name(&tex_letters("Jean-Ren\\'e Chazottes\\inst{2}")),
      "jeanrenechazottes"
    );
  }

  /// FM S2a.2 r3 item 1c: a glued label as the known labels written together, the fewest pieces; a symbol run on its
  /// own only where symbols may stand alone; none when no piece is known.
  #[test]
  fn glued_labels_part_into_known_ones() {
    let known = |ls: &[&str]| ls.iter().map(|l| l.to_string()).collect::<Vec<_>>();
    assert_eq!(
      parted_label("12", &known(&["1", "2"]), true),
      Some(vec!["1".into(), "2".into()])
    );
    assert_eq!(
      parted_label("1⋆", &known(&["1", "2"]), true),
      Some(vec!["1".into(), "⋆".into()])
    );
    assert_eq!(parted_label("1⋆", &known(&["1", "2"]), false), None);
    assert_eq!(
      parted_label("**", &known(&["*"]), false),
      Some(vec!["*".into(), "*".into()])
    );
    assert_eq!(parted_label("ab", &known(&["1"]), true), None);
    assert_eq!(parted_label("7", &known(&["7"]), true), None);
  }

  #[test]
  fn address_lines() {
    assert!(only_addresses("E-mail: {ann,bob}@x.org, cat@y.org"));
    assert!(!only_addresses("Barcelona, SPAIN. ann@x.org"));
    assert!(is_one_address("mailto:ann@x.org"));
    assert!(!is_one_address("https://x.org"));
  }
}
