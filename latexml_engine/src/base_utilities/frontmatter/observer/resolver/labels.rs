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

/// An attribute string without any `key=` pair (`role=affiliation,labelseq=affiliation` without `labelseq`).
pub(super) fn without_key(attrs: &str, key: &str) -> String {
  split_top_commas(attrs)
    .into_iter()
    .filter(|kv| kv.split_once('=').is_none_or(|(k, _)| k.trim() != key))
    .collect::<Vec<_>>()
    .join(",")
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

/// LaTeX's `\@fnsymbol` characters, the math asterisk as `*`, the kernel's formula `\|` (`∥`, 0806.2911) as `‖`: the
/// lexer's own reading of them (`events::is_fnsymbol`), which r8's decision 3 relies on the two sharing (review
/// revfms2a2_8 finding 7).
pub(super) fn is_symbol_char(c: char) -> bool { events::is_fnsymbol(c) }

/// One mark's labels (ruling 1's split): at commas and spaces; a run of one footnote symbol is one label (`**`), a
/// symbol glued to a label another (`*1` → `*`, `1`); parentheses and brackets around a label dropped (`(a)`); the
/// math asterisk `∗` read as `*`, the formula `\|`'s `∥` as `‖` (0806.2911).
pub fn mark_labels(label: &str) -> Vec<String> {
  // (a `fuzzy:<name>` label — svjour's `\institute` entry after `\at`, a name with its spaces — is one label, never
  // split: review revfms2a2 finding 2, 0807.3898, 2403.09642)
  if label.starts_with("fuzzy:") {
    return vec![label.trim().to_string()];
  }
  // (a mark holding `\ref`s is a list at its commas — `events::script_ink` writes each reference as an item of its
  // own —, each `\ref`'s key one label, never split further — its spaces and brackets are the key's —, the rest read
  // as any mark, what prints between two references no label: `\inst{\ref{ia},\ref{ib}}` and `\inst{\ref{ia}\ref{ib}}`
  // request `ref:ia` and `ref:ib`, 1707.06238, 1302.1442; `\inst{1,\ref{ib}}` `1` and `ref:ib`; `\ref{a}--\ref{c}`
  // `ref:a` and `ref:c`, its dash none)
  if label
    .split(',')
    .any(|piece| piece.trim().starts_with("ref:"))
  {
    return label
      .split(',')
      .map(str::trim)
      .filter(|piece| !piece.is_empty())
      .flat_map(|piece| {
        if piece.starts_with("ref:") {
          vec![piece.to_string()]
        } else {
          mark_labels(piece)
            .into_iter()
            .filter(|l| l.chars().any(|c| c.is_alphanumeric() || is_symbol_char(c)))
            .collect()
        }
      })
      .collect();
  }
  let mut labels = Vec::new();
  for part in label.split(|c: char| c == ',' || c == ';' || c.is_whitespace()) {
    let part = part.trim_matches(|c: char| "()[]{}".contains(c));
    // (a binding's prefix — aastex's `altaffil:` — stays on each piece: `altaffil:*` is one label, `altaffil:**`
    // another; split off, every starred label shared a bare `altaffil:` and 0911.4695's `*`/`**` notes swapped)
    if let Some((prefix, rest)) = part.split_once(':')
      && !prefix.is_empty()
      && prefix.chars().all(|c| c.is_ascii_alphabetic() || c == '@')
      && !rest.is_empty()
    {
      labels.extend(
        mark_labels(rest)
          .into_iter()
          .map(|piece| s!("{prefix}:{piece}")),
      );
      continue;
    }
    // (each character with the combining marks on it, as `parted_label` reads one: `\={\dag}`'s `†̄` and
    // `\textcircled{\dag}`'s `†⃝` are a symbol of their own, never a bare `†` and a lone mark — review rev92b_r3 B1,
    // where both requested `†` and a `\dag` author took the `\={\dag}` line —; a run is one such character repeated)
    let mut characters: Vec<String> = Vec::new();
    for c in part.chars().map(|c| match c {
      '\u{2217}' => '*',
      '\u{2225}' => '‖',
      c => c,
    }) {
      match characters.last_mut() {
        Some(character) if is_combining_mark(c) => character.push(c),
        _ => characters.push(c.to_string()),
      }
    }
    let mut run = String::new();
    let mut unit = String::new();
    let mut rest = String::new();
    for character in characters {
      if character.starts_with(is_symbol_char) {
        if !rest.is_empty() {
          labels.push(std::mem::take(&mut rest));
        }
        if !run.is_empty() && unit != character {
          labels.push(std::mem::take(&mut run));
        }
        run.push_str(&character);
        unit = character;
      } else {
        if !run.is_empty() {
          labels.push(std::mem::take(&mut run));
        }
        rest.push_str(&character);
      }
    }
    labels.extend([run, rest].into_iter().filter(|s| !s.is_empty()));
  }
  labels
}

/// The labels [`mark_labels`] parts out of one written piece at a footnote symbol glued to a label (`1*` → `1`, `*`;
/// `*1`; not `1,*` or `1 *`, written apart, nor a run of one symbol `**`): ruling 1's reading of a mark printed so
/// (2506.20360's `\inst{1*}`). None for a `\ref` list or a `fuzzy:` label, which are never parted.
pub(super) fn glued_symbol_parts(label: &str) -> Vec<String> {
  if label.starts_with("fuzzy:") || label.contains("ref:") {
    return Vec::new();
  }
  label
    .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
    .map(mark_labels)
    .filter(|labels| labels.len() >= 2)
    .flatten()
    .collect()
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
/// that fallback (OD #159, relocate.rs:42-50: a shared `affiliation:1` must not reach the first author by number) — the
/// N-th of the `live` creators, as today's numbers the authors it holds (`place.rs`'s `author:{n}`: a replaced `\author`
/// call's creators are none of them, so they shift no later label; review revfms2a2_6 finding 4, FM S2a.2 r7).
pub(super) fn label_owners(creators: &[RCreator], live: &[usize], label: &str) -> Vec<usize> {
  if let Some(n) = label.strip_prefix("author:") {
    return n
      .trim()
      .parse::<usize>()
      .ok()
      .and_then(|n| n.checked_sub(1))
      .and_then(|at| live.get(at).copied())
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
pub(in crate::base_utilities::frontmatter::observer) fn label_matches(
  request: &str,
  label: &str,
) -> bool {
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
      // (a control symbol printing its character keeps it: `\correspondingauthor{Jiacheng Meng \& Cheng Li}` names two,
      // 2008.13733; an accent command, `\'`, `\"`, `\~`: its character dropped, the letter after it kept)
      if let Some(&n) = chars.peek()
        && "&%#_".contains(n)
      {
        out.push(n);
      }
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
  // (the line's first run in a font of its own, against the affiliation line above it: the print sets the name apart —
  // but not a cell's: each cell of an alignment opens in the alignment's font, so the lexer's change of font between
  // the runs of two cells is a cell's reset as often as a switch (`$^1$\it Univ X,\\ \bf Dept Y\\ City Z` in a
  // `tabular`: "City Z" read as an author, review rev94a r5 t06))
  let mut in_cell = false;
  let own_font = events.iter().take_while(|e| !inked(e)).any(|e| {
    in_cell |= e.kind == EventKind::Layout && e.role.as_deref() == Some("cell");
    !in_cell && e.kind == EventKind::Layout && e.role.as_deref() == Some("font")
  });
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

/// The mark a Note event shows on its name (`\thanks`/`\footnote`'s `\thefootnote`, the rehearsal's `printed`), as the
/// lexer writes it last into the note's role (`thanks mark=1 shows=∗`), if one (FM S2a.2 r7, 1607.07280).
pub(super) fn note_shows(note: &Event) -> Option<&str> {
  note
    .role
    .as_deref()?
    .split_once(" shows=")
    .map(|(_, mark)| mark.trim())
    .filter(|mark| !mark.is_empty())
}

/// Whether a Mark event is one a `\footnotemark` printed (its role `footnotemark`, `footnotemark parted`).
pub(super) fn printed_by_footnotemark(event: &Event) -> bool {
  event
    .role
    .as_deref()
    .is_some_and(|r| r.starts_with("footnotemark"))
}

/// Whether a label is a footnote symbol (ruling 2).
pub(super) fn is_symbol_label(label: &str) -> bool {
  !label.is_empty() && label.chars().all(is_symbol_char)
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
    } else if let Some((group, taken)) = domain_after_group(&rest[open + 1..close], after) {
      // (the addresses of the group ending in the domain printed after its brace: `\{xxh22@mails,
      // huichen@mail\}.tsinghua.edu.cn`, 2407.18899 — each read whole; they were lost, review rev92c r2 B-3)
      out.extend(plain_addresses(&rest[..open]));
      out.extend(group);
      rest = &after[taken..];
    } else if let Some(group) = domain_inside_group(&rest[open + 1..close]) {
      // (the shared domain printed inside the group, after its last local part: `\{blaz.skrlj,senja.pollak@ijs.si\}`,
      // 1907.06944 — each local part at it, as the reader reads the line)
      out.extend(plain_addresses(&rest[..open]));
      out.extend(group);
      rest = after;
    } else {
      out.extend(plain_addresses(&rest[..=close]));
      rest = after;
    }
  }
  out.extend(plain_addresses(rest));
  out
}

/// One run of e-mail addresses printed in a text (FIDELITY_CHARTER §4: an address is a lexical object, the author it
/// spells its owner — brief 95c): its byte range — with the e-mail label word printed before it ("E-mail:", "Emails",
/// "e-mail addresses:") and the parentheses round it —, its addresses written out, one each (a braced group, a list
/// of local parts sharing the domain), and the label word as printed, when it took one (the e-mail contact's name, as
/// an e-mail line's label names its contacts: `author_block::labelled_emails`, brief 95c r2 B5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::base_utilities::frontmatter::observer) struct AddressRun {
  pub start:     usize,
  pub end:       usize,
  pub addresses: Vec<String>,
  pub label:     Option<String>,
}

/// A character of an address's local part as the reader takes it (`ann.able`, `j_doe`, `b+x`).
fn is_local_char(c: char) -> bool { c.is_ascii_alphanumeric() || "._+-".contains(c) }

/// The e-mail address runs printed in `text`, in print order ([`AddressRun`]): a braced group (`{ann,bob}@x.org`,
/// `{ann@mails, bob@mail}.x.edu`, `{ann,bob@x.org}`, `{ann@x.org, bob@y.org}` — the forms [`addresses_in`] reads), a
/// list of local parts sharing the domain, each part lowercase (`saro,borgani,tornator@oats.inaf.it`, 0810.2791;
/// `julioleite,tmariz@fis.ufal.br`, 1110.2127 — a list written without spaces, the main session's ruling; one written
/// with spaces, `hyunjun.jung, p.ruhkamp, b.busam@tum.de` (2308.10621), only where each of its parts `spells` an author
/// or the list opens the text, a parenthesis, a brace or a label — else prose before one address: `Univ X, uk, ann@x.org`),
/// and a plain address. Runs separated only by commas, semicolons, spaces or "and" are one run.
pub(in crate::base_utilities::frontmatter::observer) fn address_runs(
  text: &str,
  spells: &dyn Fn(&str) -> bool,
) -> Vec<AddressRun> {
  let domain_at = |from: usize| -> Option<(usize, String)> {
    let domain: String = text[from..]
      .chars()
      .take_while(|c| c.is_ascii_alphanumeric() || ".-".contains(*c))
      .collect();
    let domain = domain.trim_end_matches(['.', '-']);
    (domain.contains('.') && !domain.starts_with('.') && !domain.starts_with('-'))
      .then(|| (from + domain.len(), domain.to_string()))
  };
  let mut runs: Vec<AddressRun> = Vec::new();
  // (braced groups)
  let mut from = 0;
  while let Some(open) = text[from..].find('{').map(|o| from + o) {
    let Some(close) = text[open + 1..]
      .find(['{', '}'])
      .map(|c| open + 1 + c)
      .filter(|&c| text[c..].starts_with('}'))
    else {
      from = open + 1;
      continue;
    };
    let inner = &text[open + 1..close];
    let after = &text[close + 1..];
    let local = |p: &str| !p.is_empty() && p.chars().all(is_local_char);
    // (spaces before the `@` are the group's: `\{ ann, bob \} @x.org`, 2311.14729)
    let gap = after.len() - after.trim_start().len();
    let found = if let Some((end, domain)) = after
      .trim_start()
      .strip_prefix('@')
      .and_then(|_| domain_at(close + 2 + gap))
    {
      let locals: Vec<&str> = inner
        .split(|c: char| c.is_whitespace() || ",;|".contains(c))
        .filter(|p| !p.is_empty())
        .collect();
      (!locals.is_empty() && locals.iter().all(|p| local(p)))
        .then(|| (end, locals.iter().map(|l| s!("{l}@{domain}")).collect()))
    } else if let Some((group, taken)) = domain_after_group(inner, after) {
      Some((close + 1 + taken, group))
    } else if let Some(group) = domain_inside_group(inner) {
      Some((close + 1, group))
    } else if inner.contains('@') && only_addresses(inner) {
      Some((close + 1, plain_addresses(inner)))
    } else {
      None
    };
    match found {
      Some((end, addresses)) if !addresses.is_empty() => {
        runs.push(AddressRun {
          start: open,
          end,
          addresses,
          label: None,
        });
        from = end;
      },
      _ => from = close + 1,
    }
  }
  // (plain addresses and lists of local parts, outside the groups)
  let in_group = |at: usize, runs: &[AddressRun]| runs.iter().any(|r| r.start <= at && at < r.end);
  let boundary_before = |at: usize| {
    text[..at]
      .chars()
      .next_back()
      .is_none_or(|c| c.is_whitespace() || "({[<:;,".contains(c))
  };
  let mut plain: Vec<AddressRun> = Vec::new();
  for (at, _) in text.match_indices('@') {
    if in_group(at, &runs) || plain.iter().any(|r| r.start <= at && at < r.end) {
      continue;
    }
    let Some((end, domain)) = domain_at(at + 1) else {
      continue;
    };
    let mut start = at;
    while let Some(c) = text[..start].chars().next_back()
      && is_local_char(c)
    {
      start -= c.len_utf8();
    }
    while text[start..at].starts_with('.') {
      start += 1;
    }
    if start == at || !boundary_before(start) {
      continue;
    }
    // (the list of local parts before it: `a,b@x`, `a, b@x`)
    let mut locals: Vec<(usize, &str)> = Vec::new();
    let mut spaced = false;
    let mut head = start;
    loop {
      let before = text[..head].trim_end();
      let gap = head - before.len();
      let Some(comma) = before.strip_suffix(',') else {
        break;
      };
      let word_end = comma.len();
      let mut word_start = word_end;
      while let Some(c) = text[..word_start].chars().next_back()
        && is_local_char(c)
      {
        word_start -= c.len_utf8();
      }
      let word = &text[word_start..word_end];
      // (each part a local part of letters: a digit run before the address is no part of a list — `Tel:5551234,info@x.org`
      // is no `5551234@x.org`, `ORCID 0000-0002-1825-0097,ann@x.org` no `0000-…@x.org`; brief 95c r2 B3)
      if word.is_empty()
        || word.starts_with('.')
        || word.chars().any(|c| c.is_ascii_uppercase())
        || !word.chars().any(|c| c.is_ascii_alphabetic())
        || !boundary_before(word_start)
      {
        break;
      }
      spaced |= gap > 0;
      locals.push((word_start, word));
      head = word_start;
    }
    let local = &text[start..at];
    let opens = |at: usize| {
      let before = text[..at].trim_end();
      before.is_empty()
        || before.ends_with(['(', '{', '[', '<', ':'])
        || label_word_before(text, at, false).is_some()
    };
    let list_ok = !locals.is_empty()
      && (!spaced
        || locals.iter().all(|(_, w)| spells(w)) && spells(local)
        || opens(locals.last().map_or(start, |(s, _)| *s)));
    let (run_start, mut addresses) = if list_ok {
      let first = locals.last().map_or(start, |(s, _)| *s);
      (
        first,
        locals
          .iter()
          .rev()
          .map(|(_, w)| s!("{w}@{domain}"))
          .collect::<Vec<_>>(),
      )
    } else {
      (start, Vec::new())
    };
    addresses.push(s!("{local}@{domain}"));
    plain.push(AddressRun {
      start: run_start,
      end,
      addresses,
      label: None,
    });
  }
  runs.extend(plain);
  runs.sort_by_key(|r| r.start);
  // (runs separated only by separators, one run)
  let mut merged: Vec<AddressRun> = Vec::new();
  for run in runs {
    if let Some(last) = merged.last_mut()
      && text[last.end..run.start]
        .split(|c: char| c.is_whitespace() || ",;&".contains(c))
        .all(|w| w.is_empty() || w.eq_ignore_ascii_case("and"))
    {
      last.end = run.end;
      last.addresses.extend(run.addresses);
      continue;
    }
    merged.push(run);
  }
  // (its label word)
  for run in &mut merged {
    if let Some(label) = label_word_before(text, run.start, ends_a_list_item(text, run.end)) {
      run.label = Some(text[label..run.start].trim_end().to_string());
      run.start = label;
    }
  }
  // (a run out of a text only at a list boundary: brief 95c r2 B2)
  merged.retain(|run| at_list_boundary(text, run));
  // (the parentheses round it)
  for run in &mut merged {
    let before = text[..run.start].trim_end();
    let after = text[run.end..].trim_start();
    for (open, close) in [('(', ')'), ('[', ']'), ('<', '>')] {
      if before.ends_with(open) && after.starts_with(close) {
        run.start = before.len() - 1;
        run.end = text.len() - after.len() + 1;
        break;
      }
    }
  }
  merged
}

/// Whether the address run `run` of `text` stands at a list boundary, so a reading may take it out of the text (brief
/// 95c r2 B2): on one side at least the text's or a line's end (a `\n`, a line break), a list's separator (`,` `;`), a
/// bracket (an opening one or a closing one before it, a closing one after it: `(…)`, `[…]`, `<…>`) — before it, its
/// label word too ([`label_word_before`]); after it, a period ending the text or the line ([`ends_a_list_item`]) —, on
/// the other no lowercase word without a colon (r3 F5: `Univ X, see also data@x.org`, `Univ W, see sales@w.org, City
/// W`, `write to a@z.org.` are prose), and no parenthesised tag after it. A run with a plain word on both sides is
/// prose (`write to info@x.org or data@x.org for data`), and one a tag follows is tagged (`ann@y.org (Ann), bob@y.org
/// (Bob)`, `zhanghuamaths@163.com(H. Zhang)`, 0811.1419): it stays where it prints, as today's prints it (the
/// fail-safe). One side is enough when the other is no such word: `Correspondence: {…}@westlake.edu.cn` (2601.07312),
/// `SPAIN. {argimiro.arratia, …}@upc.edu` (2509.12185), `KTH, Sweden sbroome,…@kth.se` (2112.12175), `…, Russia.
/// …@…` (2112.04350).
fn at_list_boundary(text: &str, run: &AddressRun) -> bool {
  let blank = |c: char| c.is_whitespace() && c != '\n';
  let before = text[..run.start].trim_end_matches(blank);
  let after = text[run.end..].trim_start_matches(blank);
  let before_ok = run.label.is_some()
    || before.is_empty()
    || before.ends_with(['\n', ',', ';', '(', '[', '<', ')', ']', '>']);
  let after_ok = ends_a_list_item(text, run.end);
  // (a lowercase word without a colon beside it: prose's)
  let prose = |word: Option<&str>| {
    word.is_some_and(|w| w.chars().next().is_some_and(|c| c.is_lowercase()) && !w.ends_with(':'))
  };
  let word_before = before
    .rsplit(char::is_whitespace)
    .next()
    .filter(|w| !w.is_empty());
  let word_after = after
    .split(char::is_whitespace)
    .next()
    .filter(|w| !w.is_empty());
  let tagged = after.starts_with(['(', '[']);
  ((before_ok && (after_ok || !prose(word_after))) || (after_ok && !prose(word_before))) && !tagged
}

/// Whether an address run ending at `end` of `text` ends a list's item there: the text's or a line's end, a separator
/// (`,` `;`), a closing bracket, or a period ending the text or the line.
fn ends_a_list_item(text: &str, end: usize) -> bool {
  let blank = |c: char| c.is_whitespace() && c != '\n';
  let after = text[end..].trim_start_matches(blank);
  after.is_empty()
    || after.starts_with(['\n', ',', ';', ')', ']', '>'])
    || after.strip_prefix('.').is_some_and(|rest| {
      rest.trim_start_matches(blank).is_empty() || rest.trim_start_matches(blank).starts_with('\n')
    })
}

/// Where the e-mail label word printed just before `at` starts ("E-mail:", "Emails", "e-mail addresses:", "Email
/// address"), if one that is a label: a colon after it, or nothing before it on its line — the text's start or a line
/// break (brief 95c r2 S4: `see …, mail ann@x.org now` is prose, its "mail" no label) —, or, `ends_item` (the run after
/// it ends a list's item, [`ends_a_list_item`]), a list's separator or a sentence's period before it (r3 F4: `Univ X,
/// E-mail ann@x.org`, `Univ Y. Email bob@y.org`; `\texttt{mail ann@x.org now}` keeps its "mail").
fn label_word_before(text: &str, at: usize, ends_item: bool) -> Option<usize> {
  let start = mail_word_before(text, at)?;
  let colon = text[..at].trim_end().ends_with(':');
  let line = text[..start].rfind('\n').map_or(0, |p| p + 1);
  let lead = text[line..start].trim();
  let listed = ends_item && lead.ends_with([',', ';', '.', '(', '[', ')', ']']);
  (colon || lead.is_empty() || listed).then_some(start)
}

/// Where the e-mail word printed just before `at` starts ("E-mail:", "Emails", "e-mail addresses:", "Email address"; a
/// colon after it or none; `author_block::is_email_label`'s words, the e-mail line's: brief 95c r3 N1), if one.
fn mail_word_before(text: &str, at: usize) -> Option<usize> {
  let before = text[..at].trim_end();
  let before = before.strip_suffix(':').unwrap_or(before).trim_end();
  let word_start = |end: usize| {
    text[..end]
      .rfind(|c: char| !(c.is_alphabetic() || c == '-'))
      .map_or(0, |p| {
        p + text[p..].chars().next().map_or(1, char::len_utf8)
      })
  };
  let start = word_start(before.len());
  if start == before.len() {
    return None;
  }
  let word = &text[start..before.len()];
  if is_email_label(word) {
    return Some(start);
  }
  // (an address word after it: "E-mail address", "e-mail addresses")
  let prev = text[..start].trim_end();
  let prev_start = word_start(prev.len());
  (prev_start < prev.len() && is_email_label(&text[prev_start..before.len()])).then_some(prev_start)
}

/// The addresses printed in `text` as the reading takes them out of an affiliation entry
/// (`Resolver::take_out_entry_addresses`, brief 95c): every address of its runs (`address_runs`, a list part spelling
/// one of `names` read as a part), written out, and the text the cut leaves (`strip_address_runs`) — what `ab_audit
/// fmshadow` reads today's uncut entry, or today's e-mail printing several addresses, by.
pub fn address_take_out(text: &str, names: &[&str]) -> (Vec<String>, String) {
  let spells = |local: &str| spelled_name(names, &s!("{local}@x.org")).is_some();
  let runs = address_runs(text, &spells);
  let addresses = runs.iter().flat_map(|r| r.addresses.clone()).collect();
  (addresses, strip_address_runs(text, &runs))
}

/// The addresses of a text that prints addresses alone, two or more — a list (`ann@x.org, bob@y.org`,
/// `tuhin@…;rajib@…`), a group or a list sharing the domain (`{ann,bob}@x.org`, `julioleite,tmariz@fis.ufal.br`) —,
/// written out ([`address_runs`]); `None` for one address, a text with other words beside them, or a label word before
/// them (`Emails: {ann,bob}@x.org`, which the writer keeps whole: `plan`'s group write-out).
pub(in crate::base_utilities::frontmatter::observer) fn listed_addresses(
  text: &str,
  spells: &dyn Fn(&str) -> bool,
) -> Option<Vec<String>> {
  let runs = address_runs(text, spells);
  let labelled = text
    .split(|c: char| c.is_whitespace() || ",;:".contains(c))
    .any(|w| !w.is_empty() && is_email_label(w));
  let addresses: Vec<String> = runs.iter().flat_map(|r| r.addresses.clone()).collect();
  (addresses.len() >= 2 && !labelled && strip_address_runs(text, &runs).is_empty())
    .then_some(addresses)
}

/// The byte ranges of `text` the address runs `runs` take out ([`address_runs`]), with the separators their place
/// leaves stranded, each read within its line (a `\n`, a line break, ends one: brief 95c r2 N2): a run ending its line
/// takes the separators before it (`Trieste, Italy, (ann@x.org)` leaves `Trieste, Italy`; cond-mat0607219's `FAX:
/// 951-827-2425; email: rlake@…` its `FAX: 951-827-2425` before the break), a run opening it those after it, a run
/// that is its line the line's break too, a run between two separators the one after it (`FAX: 1, email: a@x.org, Jan.
/// 4` leaves `FAX: 1, Jan. 4`), a run between a word and a separator the spaces before it (`Univ X (a@x.org), City`
/// leaves `Univ X, City`); and a cut between two spaces one of them (N1: `Univ Y, ann@x.org, Phone` leaves `Univ Y,
/// Phone`).
pub(in crate::base_utilities::frontmatter::observer) fn address_cuts(
  text: &str,
  runs: &[AddressRun],
) -> Vec<(usize, usize)> {
  const SEPARATORS: &[char] = &[',', ';', ':', '-', '\u{2013}', '\u{2014}', '/'];
  let blank = |c: char| c.is_whitespace() && c != '\n';
  let stray = |c: char| blank(c) || SEPARATORS.contains(&c);
  let mut cuts: Vec<(usize, usize)> = Vec::new();
  for run in runs {
    let (mut start, mut end) = (run.start, run.end);
    let line_start = text[..start].rfind('\n').map_or(0, |p| p + 1);
    let line_end = text[end..].find('\n').map_or(text.len(), |p| end + p);
    let before = text[line_start..start].trim_end_matches(blank);
    let after = text[end..line_end].trim_start_matches(blank);
    if after.chars().all(|c| stray(c) || c == '.') {
      end = line_end;
      start = line_start + text[line_start..start].trim_end_matches(stray).len();
      // (the whole line: its break too)
      if start == line_start {
        if line_end < text.len() {
          end = line_end + 1;
        } else if line_start > 0 {
          start = line_start - 1;
        }
      }
    } else if before.chars().all(stray) {
      start = line_start;
      end = line_end - text[end..line_end].trim_start_matches(stray).len();
    } else if before.ends_with([',', ';']) && after.starts_with([',', ';']) {
      end = line_end - after.len() + 1;
    } else if after.starts_with([',', ';']) {
      start = line_start + before.len();
    } else if before.ends_with([',', ';']) && after.starts_with([')', ']', '>']) {
      start = line_start + before.trim_end_matches(stray).len();
    }
    // (one space of two)
    if text[..start].ends_with(blank) && text[end..].starts_with(blank) {
      end = text.len() - text[end..].trim_start_matches(blank).len();
    }
    match cuts.last_mut() {
      Some(last) if last.1 >= start => last.1 = last.1.max(end),
      _ => cuts.push((start, end)),
    }
  }
  cuts
}

/// `text` with the address runs `runs` taken out ([`address_cuts`]), its spaces collapsed.
pub(in crate::base_utilities::frontmatter::observer) fn strip_address_runs(
  text: &str,
  runs: &[AddressRun],
) -> String {
  let mut out = String::new();
  let mut at = 0;
  for (start, end) in address_cuts(text, runs) {
    out.push_str(&text[at..start]);
    out.push(' ');
    at = end;
  }
  out.push_str(&text[at..]);
  out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A braced group of addresses each missing the domain the text prints after the group's brace (`inner` =
/// `xxh22@mails, huichen@mail`, `after` = `.tsinghua.edu.cn, …`): each address with that domain, and the length of
/// `after` the domain takes; `None` for any other group (a piece with no `@` or two, or no domain after the brace).
fn domain_after_group(inner: &str, after: &str) -> Option<(Vec<String>, usize)> {
  let rest = after.strip_prefix('.')?;
  let domain: String = rest
    .chars()
    .take_while(|c| !c.is_whitespace() && !",;()<>{}".contains(*c))
    .collect();
  let domain = domain.trim_end_matches('.');
  if domain.is_empty() {
    return None;
  }
  let pieces: Vec<&str> = inner.split(',').map(str::trim).collect();
  let whole = |p: &str| {
    p.split_once('@').is_some_and(|(local, sub)| {
      !local.is_empty()
        && !sub.is_empty()
        && !p.contains(|c: char| c.is_whitespace())
        && !sub.contains('@')
    })
  };
  if !pieces.iter().all(|p| whole(p)) {
    return None;
  }
  Some((
    pieces.iter().map(|p| format!("{p}.{domain}")).collect(),
    1 + domain.len(),
  ))
}

/// A braced group of local parts whose last holds the one `@` and the shared domain (`blaz.skrlj,senja.pollak@ijs.si`),
/// written out: each local part at that domain; `None` for any other group.
fn domain_inside_group(inner: &str) -> Option<Vec<String>> {
  let pieces: Vec<&str> = inner.split(',').map(str::trim).collect();
  let (last, locals) = pieces.split_last()?;
  let (local, domain) = last.split_once('@')?;
  let plain = |p: &str| !p.is_empty() && !p.contains(|c: char| c == '@' || c.is_whitespace());
  if locals.is_empty()
    || !locals.iter().all(|p| plain(p))
    || !plain(local)
    || !domain.contains('.')
    || !plain(domain)
  {
    return None;
  }
  Some(
    locals
      .iter()
      .chain([&local])
      .map(|l| format!("{l}@{}", domain.trim_end_matches('.')))
      .collect(),
  )
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
/// A link to an address (`mailto:`) is an e-mail, whatever it prints (`\href{mailto:tsamak@clemson.edu}{tsamak}`,
/// 2309.10007 — review revfms2a2_8 finding 1, FM S2a.2 r9).
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
  if event.target.as_deref().and_then(mailto_address).is_some() {
    return ("email".into(), event.text.clone());
  }
  (
    contact_role(event.role.as_deref(), &event.text),
    event.text.clone(),
  )
}

/// A Contact event's link target, where the link points ([`Assoc::target`]): an `\href`'s first argument, a `\url`'s
/// text, the address of a `mailto:` link (its scheme and a `?subject=…` query aside, [`mailto_address`]); none for a
/// contact no link printed (review revfms2a2_8 finding 1, FM S2a.2 r9: the anchor text alone was kept —
/// `\href{https://academics.aut.ac.nz/robin.hankin}{work}` read "work", 2210.15991).
pub(super) fn link_target(event: &Event) -> Option<String> {
  let target = event.target.as_deref()?.trim();
  if target.is_empty() {
    return None;
  }
  Some(mailto_address(target).unwrap_or(target).to_string())
}

/// The address a `mailto:` link points to, if it is one (`mailto:ann@x.org?subject=Hi` "ann@x.org").
pub(super) fn mailto_address(target: &str) -> Option<&str> {
  let target = target.trim();
  let rest = target
    .get(..7)
    .filter(|scheme| scheme.eq_ignore_ascii_case("mailto:"))
    .map(|_| &target[7..])?;
  let address = rest.split('?').next().unwrap_or("").trim();
  (!address.is_empty()).then_some(address)
}

/// A link's text without its scheme and a `www.` host prefix: what a name may spell in it (`http://www.pengxu.net/` is
/// Peng Xu's, 2001.02600).
pub(super) fn link_letters(link: &str) -> &str {
  let link = link.trim();
  let link = link.split_once("://").map_or(link, |(scheme, rest)| {
    if scheme.chars().all(char::is_alphabetic) {
      rest
    } else {
      link
    }
  });
  link.strip_prefix("www.").unwrap_or(link)
}

/// The part of a link that says whose page it is ([`link_letters`]'s host and path, a query or a fragment aside): its
/// last path segment, an `index.*` or `*.html` page after it aside; its host when its path holds nothing else
/// (main-session decision 1 on FM S2a.2 r7, review revfms2a2_7 finding 3). A homepage spells its owner there —
/// `sites.google.com/view/<name>`, `tu.berlin/disco/klimm`, `…/MarcinPitera`, `portal.research.lu.se/en/persons/<name>`,
/// `insti.physics.sunysb.edu/~siegel/plan.html`, `www.pengxu.net/` (2301.03638, 1409.7028, 2601.07017, 1603.02588,
/// 2001.02600) —; a repository or a file under an account does not: `github.com/<user>/<repo>`, `…/resolution.git`,
/// `<user>.github.io/<project>`, `…/<name>/accvideo.mp4` (2008.12970, 2511.08416, 2103.05251, 2503.02954, 2509.01952).
/// A page named with a leading `@` (`medium.com/@jjones`) is the name after it: the owner spelling splits an address at
/// its `@`, which left the page no letters to spell (review revfms2a2_9 finding 4).
pub(super) fn link_page(link: &str) -> &str { link_segment(link).trim_start_matches('@') }

/// [`link_page`]'s segment, as the link writes it.
fn link_segment(link: &str) -> &str {
  let rest = link_letters(link);
  let rest = rest.split(['?', '#']).next().unwrap_or("");
  let mut parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
  if parts.len() >= 2
    && let Some(&last) = parts.last()
  {
    let page = last.to_ascii_lowercase();
    if page.starts_with("index.") {
      parts.pop();
    } else if let Some(stem) = [".html", ".htm"]
      .iter()
      .find(|ext| page.ends_with(*ext))
      .map(|ext| &last[..last.len() - ext.len()])
    {
      // (a page right under the host is the page that spells its owner, `host/jsmith.html`; one under a segment is that
      // segment's, `…/~siegel/plan.html` — review revfms2a2_8 finding 5, FM S2a.2 r9)
      if parts.len() == 2 && !stem.is_empty() {
        return stem;
      }
      parts.pop();
    }
  }
  parts.last().copied().unwrap_or("")
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

/// The pieces of the text a document's note prints before an address, each a name it may give
/// (`Resolver::note_named`): parted at `,`, `;`, `:`, `/`, `&` and the words "and" and "or". A label is a piece of its
/// own that names no one ("Corresponding author: A. Able" gives "Corresponding author" and "A. Able"; "A. Able:
/// corresponding author" the same two, review rev92a_r3 S1). The words part the text, so a creator surnamed Or or And
/// (Yuval Or) is never named: the note fails to the block.
pub(super) fn note_pieces(letters: &str) -> Vec<String> {
  let mut pieces = Vec::new();
  for part in letters.split([',', ';', ':', '/', '&']) {
    let mut piece: Vec<&str> = Vec::new();
    for word in part.split_whitespace() {
      if matches!(word.to_lowercase().as_str(), "and" | "or") {
        pieces.push(piece.join(" "));
        piece.clear();
      } else {
        piece.push(word);
      }
    }
    pieces.push(piece.join(" "));
  }
  pieces
    .into_iter()
    .map(|p| p.trim().to_string())
    .filter(|p| !p.is_empty())
    .collect()
}

/// The words of a name, each folded ([`fold_name`]) with whether it is an initial: parted at spaces and hyphens, and
/// after an initial's period (`A.B.` two initials; `J.-P.` and `Jean-Pierre` both two words); a word is an initial when
/// it folds to one letter.
pub(super) fn name_words(name: &str) -> Vec<(String, bool)> {
  name
    .split(|c: char| c.is_whitespace() || c == '-' || c == '.')
    .map(fold_name)
    .filter(|w| !w.is_empty())
    .map(|w| {
      let initial = w.chars().count() == 1;
      (w, initial)
    })
    .collect()
}

/// Whether the words of a note's piece are a creator's name, every word agreeing (review rev92a_r2 B2): the last is
/// the creator's surname, whole; each before it one of the creator's given names, in order, equal or its initial (`J.
/// Smith` is not Adam Smith; `B. Bitsch` is Bertram Bitsch, 1411.3255; `O. Kochukhov` Oleg Kochukhov, 0903.3512). A
/// piece with any other word ("the editor of Bob Baker") is no creator's name.
pub(super) fn names_agree(piece: &[(String, bool)], creator: &[(String, bool)]) -> bool {
  let (Some(((surname, false), given)), Some(((own_surname, _), own_given))) =
    (piece.split_last(), creator.split_last())
  else {
    return false;
  };
  if surname != own_surname {
    return false;
  }
  let agree = |(a, a_initial): &(String, bool), (b, b_initial): &(String, bool)| {
    a == b || ((*a_initial || *b_initial) && a.chars().next() == b.chars().next())
  };
  let mut own = own_given.iter();
  given.iter().all(|word| own.any(|o| agree(word, o)))
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

/// `label` as two or more labels of `known` written together (`12` → `1`, `2`); with `symbols`, a run of characters that
/// are no letter or digit may stand alone too (`1⋆` → `1`, `⋆`), and so may one letter (`1a` → `1`, `a`: the letter a
/// note's mark, `$^{1{\rm a}}$`, 1904.02582), as long as one piece is known. The most known pieces win, then the fewest
/// pieces: a symbol run spelling known labels is parted into them before it stands alone as one piece (`†♮♯` under
/// entries `†`, `♮`, `♯`: `$^{\dagger\natural\sharp}$`, 1004.1402). A combining mark is part of the character before
/// it, never parted from it (`\~{x}`'s `x̃` is no `x` and a `̃`, review rev92b_r2 B1). A run of digits is one number,
/// parted into its digits only under single-digit numbering — every number `known` holds one digit, as the names' side
/// of `part_glued_labels` reads a number —: BESIII's sub-lettered `23A` under entries numbered 1 to 70 is `23` and `A`,
/// never `2`, `3` and `A` (2007.02542, 36 sub-lettered authors each the institutes 2 and 3; 1507.02068's `48A`,
/// 2411.07730, 2606.14105; review rev98e B1). A label holding a backslash is
/// a call's label as its source spells it, not as it prints: only what it spells before its first command is read, as
/// the labels that part (authblk's `\author[3\authfn{2}]{…}`, which prints `3†`, is `3` — 2405.16051: parted whole,
/// its command's letters were labels and its argument the `2` of `\affil[1,2,4]`; `\author[1\thanks{…}]{…}` is `1`,
/// 2202.05736, 2410.10922).
pub(super) fn parted_label(label: &str, known: &[String], symbols: bool) -> Option<Vec<String>> {
  if let Some(at) = label.find('\\') {
    let spelled = label[..at].trim();
    return match parted_label(spelled, known, symbols) {
      Some(parts) => Some(parts),
      None => known
        .iter()
        .any(|k| k == spelled)
        .then(|| vec![spelled.to_string()]),
    };
  }
  // (each character with the combining marks on it)
  let chars: Vec<(usize, char)> = label
    .char_indices()
    .filter(|(at, c)| *at == 0 || !is_combining_mark(*c))
    .collect();
  let n = chars.len();
  if n < 2 {
    return None;
  }
  let at = |i: usize| chars.get(i).map_or(label.len(), |(b, _)| *b);
  // (a cut between two digits only under single-digit numbering)
  let single_digit = known
    .iter()
    .filter(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_digit()))
    .all(|k| k.len() == 1);
  let digit_at = |i: usize| chars.get(i).is_some_and(|(_, c)| c.is_ascii_digit());
  let cuts = |i: usize| single_digit || i == 0 || i == n || !(digit_at(i - 1) && digit_at(i));
  // best[i]: the fewest pieces covering label[..i], with how many are known, and the piece before
  let mut best: Vec<Option<(usize, usize, usize)>> = vec![None; n + 1];
  best[0] = Some((0, 0, 0));
  for i in 0..n {
    let Some((count, hits, _)) = best[i] else {
      continue;
    };
    for j in (i + 1..=n).filter(|&j| cuts(j)) {
      let piece = &label[at(i)..at(j)];
      let is_known = known.iter().any(|k| k == piece);
      let is_symbol = symbols && piece.chars().all(|c| !c.is_alphanumeric());
      let mut bases = piece.chars().filter(|c| !is_combining_mark(*c));
      let is_letter =
        symbols && bases.next().is_some_and(char::is_alphabetic) && bases.next().is_none();
      if !is_known && !is_symbol && !is_letter {
        continue;
      }
      let candidate = (count + 1, hits + usize::from(is_known), i);
      if best[j].is_none_or(|(c, h, _)| {
        (std::cmp::Reverse(candidate.1), candidate.0) < (std::cmp::Reverse(h), c)
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
