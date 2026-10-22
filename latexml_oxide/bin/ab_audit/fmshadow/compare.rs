//! The comparison of the shadow's tuples with the XML's, each disagreement with its kind, its rule and whether it is
//! classified (an adopted ruling's change tag, or a PDF-confirmed XML-wrong class: `classes.rs`).

use super::*;

/// One disagreement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disagreement {
  pub kind:       &'static str,
  pub rule:       String,
  pub classified: bool,
  /// the [`XML_WRONG`] class that classified it, when one did
  pub class:      Option<&'static str>,
  pub detail:     String,
}

/// The rule tags whose purpose is a change of today's output — an adopted ruling's (FIDELITY_CHARTER §4, 2026-10-21;
/// the S2a.2 brief's one contact per print event and emptied notes): a disagreement a rule holding one of them decided
/// is classified.
///
/// - ruling 1, the marks shown on a name are its requests: `ruling1:request` (a labelled line's entry, or a note's
///   legend opening with its Mark event), `ruling1:fuzzy-label`, `ruling1:thenote` (authblk's printed number), `ruling1:parted` (a glued mark read as the marks it is: `12`
///   under entries `1` and `2`), `api:leading-mark` (an API entry's leading Mark
///   events: by construction only a real Mark, `resolver/api.rs` `api`), the `\ref`-mark keys (`api:ref-label`, `ref:`);
/// - ruling 2, a footnote-symbol line labels the line for its carriers: `ruling2:symbol-line`, `ruling2:uncarried`;
/// - the corresponding author (charter §4's e-mail owner order): `owner:corresponding`;
/// - one contact per print event (`ruling3:one-per-print`): counted only where the XML holds the pieces joined into one
///   item (an `overlaps` rule), the change the ruling makes — the owner of each address is its other tag's;
/// - an emptied note dropped, a note's inner contacts its contacts, `\thanks{}` kept: `ruling4:`;
/// - a url whose text is one e-mail address is an e-mail (`role:url-as-email`: the S2a.2 r2 brief's item 8, adopted);
/// - a list with one address per author that every spelled address agrees with is in author order (`owner:order`:
///   charter §4's owner order, 2026-10-21, OD #805 — for every class, not only the sequenced lists);
/// - several addresses an entry's e-mail call prints (a braced `{a,b,c}@x` group in an `\institute`/`\affil` entry)
///   are written out, each the author's it spells, an unspelled one the block's (`ruling3:written-out`: the main
///   session's decision 3 of 2026-10-22, user rulings 2026-10-20 #4, #8, #10).
///
/// Elimination (`owner:elimination`, the one address left to the one name left) is NOT a change: today's output
/// applies it too (OXIDIZED_DESIGN_DIVERGENCES #52's e-mail lines, `owners_by_elimination`, under user ruling R10
/// 2026-10-20, 77p r10;
/// FIDELITY_CHARTER §4's owner order is silent on it), so its disagreements are the shadow's to explain.
pub(super) const CHANGE_TAGS: &[&str] = &[
  "ruling1:request",
  "ruling1:fuzzy-label",
  "ruling1:thenote",
  "ruling1:parted",
  "role:url-as-email",
  "owner:order",
  "api:leading-mark",
  "api:ref-label",
  "ref:",
  "ruling2:symbol-line",
  "ruling2:uncarried",
  "owner:corresponding",
  "ruling3:one-per-print",
  "ruling3:written-out",
  "ruling4:",
];

/// Whether a shadow rule holds one of [`CHANGE_TAGS`] (`ruling3:one-per-print` only on a joined item's `overlaps`
/// rule).
pub fn is_change_rule(rule: &str) -> bool {
  let joined = rule.starts_with("overlaps ");
  rule.split_whitespace().any(|tag| {
    CHANGE_TAGS.iter().any(|c| tag.starts_with(c)) && (joined || tag != "ruling3:one-per-print")
  })
}

/// A disagreement's context for the [`XML_WRONG`] classes.
pub struct Context<'a> {
  pub kind:   &'static str,
  pub shadow: &'a Tuples,
  pub xml:    &'a Tuples,
  /// the disagreement's detail (`<name>: shadow-only email\t<text>`)
  pub detail: &'a str,
  /// the shadow rule that decided it (`none` when none did)
  pub rule:   &'a str,
}

pub(super) fn multiset(items: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
  let mut m = BTreeMap::new();
  for item in items.filter(|i| !i.is_empty()) {
    *m.entry(item).or_default() += 1;
  }
  m
}

/// The items of `a` not in `b` (as multisets).
pub(super) fn minus(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> Vec<String> {
  a.iter()
    .flat_map(|(k, n)| std::iter::repeat_n(k.clone(), n.saturating_sub(*b.get(k).unwrap_or(&0))))
    .collect()
}

/// Every rule map of the shadow: each creator's, the block's, and what the rulings dropped.
pub(super) fn rule_maps(shadow: &Tuples) -> impl Iterator<Item = &BTreeMap<String, String>> {
  shadow
    .creators
    .iter()
    .map(|c| &c.rules)
    .chain([&shadow.block_rules, &shadow.dropped])
}

/// The shadow rule that decided an item of `kind` with normalised text `text`, anywhere in the shadow (a creator's or the
/// block's): the item of that text (`moved …`), else — when the text is two or more of the shadow's items of that kind
/// joined, as today's one contact of two symbol lines `†Univ A‡Univ B` is the shadow's two — the rule of its first piece
/// (`overlaps …`), else one line of an item the shadow joined from continuation lines (`part …`, `line:continued`). No
/// looser overlap: any other text that only shares a part with an item is the shadow's to explain.
pub(super) fn rule_of(shadow: &Tuples, kind: &str, text: &str) -> Option<String> {
  let key = format!("{kind}\t{text}");
  if let Some(rule) = rule_maps(shadow).find_map(|rules| rules.get(&key)) {
    return Some(format!("moved {rule}"));
  }
  let prefix = format!("{kind}\t");
  let pieces: Vec<(&str, &str)> = rule_maps(shadow)
    .flat_map(|rules| {
      rules.iter().filter_map(|(k, rule)| {
        k.strip_prefix(prefix.as_str())
          .filter(|p| !p.is_empty() && *p != text)
          .map(|p| (p, rule.as_str()))
      })
    })
    .collect();
  // (the text as a sequence of two or more pieces: the first piece's rule, the shortest such sequence)
  let n = text.len();
  let mut best: Vec<Option<(usize, usize, &str)>> = vec![None; n + 1];
  best[0] = Some((0, 0, ""));
  for at in 0..n {
    let Some((count, ..)) = best[at] else {
      continue;
    };
    // (a piece after the first may follow the word "and" the XML kept between them: `Berkeley and $^2$National
    // Bureau`, 1810.07216)
    let words: Vec<usize> = if at > 0 && text[at..].starts_with("and") {
      vec![at, at + 3]
    } else {
      vec![at]
    };
    // (or the label the XML kept printed before it, the shadow's pieces split by those marks: `1INFN …2Dipartimento
    // …`, 1211.5032 — one or two digits, or a letter `a`-`h`; after the "and" too: `1Dept A and2Dept B`, 2112.13115)
    let mut starts: Vec<usize> = Vec::new();
    for word in words {
      starts.push(word);
      let rest = &text.as_bytes()[word..];
      let digits = rest
        .iter()
        .take(2)
        .take_while(|b| b.is_ascii_digit())
        .count();
      starts.extend((1..=digits).map(|k| word + k));
      if digits == 0 && rest.first().is_some_and(|b| (b'a'..=b'h').contains(b)) {
        starts.push(word + 1);
      }
    }
    for start in starts {
      for (piece, rule) in &pieces {
        let end = start + piece.len();
        if end <= n
          && text[start..].starts_with(piece)
          && best[end].is_none_or(|(c, ..)| c > count + 1)
        {
          let first = if at == 0 {
            *rule
          } else {
            best[at].map_or("", |b| b.2)
          };
          best[end] = Some((count + 1, at, first));
        }
      }
    }
  }
  match best[n] {
    Some((count, _, first)) if count >= 2 => Some(format!("overlaps {first}")),
    // (or one line of a shadow item that its continuation lines joined into one affiliation (`line:continued`): today's
    // one contact a line where the shadow reads the lines as one entry — `$\dagger$ Laboratory of Physical and
    // Structural Biology, NICHD, Bld. 9, Rm. 1E116 \\ National Institutes of Health, …` under the `†` carriers once the
    // lexer reads `$\diamond$` as a mark, 0907.1243, FM S2a.2 r7 — the joined item's rule, `part …`)
    _ => pieces
      .iter()
      .filter(|(piece, rule)| {
        rule.contains("line:continued")
          && text.len() >= 8
          && piece.len() > text.len()
          && piece.contains(text)
      })
      .map(|(_, rule)| format!("part {rule}"))
      .next(),
  }
}

/// The shadow rule that decided an XML note's text ([`rule_of`]): a note of the shadow's, or an entry's legend it read as
/// that label's affiliation (`note:legend`, the note opening with a mark its requesters show: `\thanks{$\star$
/// Communications and Information Theory Group, …}` under the `⋆` authors, 1803.05754, 2607.19263 — FM S2a.2 r7).
pub(super) fn note_rule_of(shadow: &Tuples, text: &str) -> Option<String> {
  rule_of(shadow, "note", text)
    .or_else(|| rule_of(shadow, "affiliation", text).filter(|rule| rule.contains("note:legend")))
}

/// The shadow rule that decided an XML affiliation's text ([`rule_of`]): an affiliation of the shadow's, or a labelled
/// line's entry it read as a legend, a note (`ruling1:request`, `ruling2:symbol-line`: `$^{\$}$These authors contributed
/// equally to this work.` under the names marked `$`, 1910.10432 — FM S2a.2 r7, a legend's mark a symbol).
pub(super) fn affiliation_rule_of(shadow: &Tuples, text: &str) -> Option<String> {
  rule_of(shadow, "affiliation", text).or_else(|| {
    rule_of(shadow, "note", text)
      .filter(|rule| rule.contains("ruling1:request") || rule.contains("ruling2:symbol-line"))
  })
}

/// The shadow rule that explains an XML e-mail printing several addresses (`raw`, as the XML holds it: today's whole
/// `{ann,bob,info}@x.org` group on every author of an `\institute` entry) by the shadow's addresses written out of it
/// (`ruling3:written-out`, decision 3 of 2026-10-22): every address of it is a shadow e-mail so tagged, somewhere — the
/// first one's rule (`overlaps …`).
pub(super) fn written_out_rule(shadow: &Tuples, raw: &str) -> Option<String> {
  let addresses = fm_addresses_in(raw);
  if addresses.len() < 2 {
    return None;
  }
  let rules: Vec<&String> = addresses
    .iter()
    .map(|a| {
      let key = format!("email\t{}", norm(a));
      rule_maps(shadow)
        .find_map(|rules| rules.get(&key))
        .filter(|rule| rule.contains("ruling3:written-out"))
    })
    .collect::<Option<_>>()?;
  rules.first().map(|rule| format!("overlaps {rule}"))
}

/// A normalised text without the ORCID iDs it holds (`0000000283879887`: 16 characters opening with `000`, the last a
/// digit or `x`, no digit on either side).
pub(super) fn without_orcid_ids(text: &str) -> String {
  let chars: Vec<char> = text.chars().collect();
  let is_id = |at: usize| {
    at + 16 <= chars.len()
      && chars[at..at + 3].iter().all(|c| *c == '0')
      && chars[at + 3..at + 15].iter().all(char::is_ascii_digit)
      && (chars[at + 15].is_ascii_digit() || chars[at + 15] == 'x')
      && (at == 0 || !chars[at - 1].is_ascii_digit())
      && chars.get(at + 16).is_none_or(|c| !c.is_ascii_digit())
  };
  let mut out = String::with_capacity(text.len());
  let mut at = 0;
  while at < chars.len() {
    if is_id(at) {
      at += 16;
    } else {
      out.push(chars[at]);
      at += 1;
    }
  }
  out
}

/// Whether `rest` is the marks `marks` written one after another (`12` the marks `1`, `2`; `ab`).
pub(super) fn is_marks(rest: &str, marks: &[String]) -> bool {
  if rest.is_empty() {
    return true;
  }
  marks
    .iter()
    .any(|m| !m.is_empty() && rest.starts_with(m.as_str()) && is_marks(&rest[m.len()..], marks))
}

/// The XML's creator names, normalised, a name whose marks the XML holds as text (not an `ltx:sup`: `R. Vasile 1`,
/// 0807.4407; `Manoranjan Kuma r 1`, 1309.7542; `Krzysztof Pawłowski 2`, 1511.09260) read as the shadow's name it is
/// with that name's marks after it.
pub(super) fn xml_names(shadow: &Tuples, xml: &Tuples) -> Vec<String> {
  xml
    .creators
    .iter()
    .map(|c| {
      let x = norm(&c.name);
      shadow
        .creators
        .iter()
        .find_map(|sc| {
          let n = norm(&sc.name);
          (x != n && !n.is_empty() && x.starts_with(&n) && is_marks(&x[n.len()..], &sc.marks))
            .then_some(n)
        })
        .unwrap_or(x)
    })
    .collect()
}

/// Compare the shadow's tuples with the XML's.
pub fn compare(shadow: &Tuples, xml: &Tuples) -> Vec<Disagreement> {
  let mut out: Vec<(&'static str, Option<String>, String)> = Vec::new();
  let mut push =
    |kind: &'static str, rule: Option<String>, detail: String| out.push((kind, rule, detail));
  let sn: Vec<String> = shadow.creators.iter().map(|c| norm(&c.name)).collect();
  let xn: Vec<String> = xml_names(shadow, xml);
  // the creators matched by name, in order (a longest common subsequence)
  let pairs = align(&sn, &xn);
  if pairs.len() != sn.len() || pairs.len() != xn.len() {
    let first = (0..sn.len().max(xn.len()))
      .find(|&i| sn.get(i) != xn.get(i))
      .unwrap_or(0);
    let rule = shadow
      .creators
      .get(first)
      .and_then(|c| c.rules.get("name").cloned());
    let show = |t: &Tuples| {
      t.creators
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join(" | ")
    };
    push(
      "name-split",
      rule,
      format!("shadow [{}] xml [{}]", show(shadow), show(xml)),
    );
  }
  for (i, j) in pairs {
    let (sc, xc) = (&shadow.creators[i], &xml.creators[j]);
    let sa = multiset(sc.affiliations.iter().map(|a| norm(a)));
    let xa = multiset(xc.affiliations.iter().map(|a| norm(a)));
    for text in minus(&sa, &xa) {
      push(
        "affiliation",
        sc.rules.get(&format!("affiliation\t{text}")).cloned(),
        format!("{}: shadow-only {text}", sc.name),
      );
    }
    for text in minus(&xa, &sa) {
      push(
        "affiliation",
        affiliation_rule_of(shadow, &text),
        format!("{}: xml-only {text}", sc.name),
      );
    }
    for (kind, emails) in [("contact-owner", true), ("contact", false)] {
      // (an address with its label glued, `Email:jingyuan@xmu.edu.cn`, is the address: 2108.12329)
      let address = |r: &str, t: &str| {
        let t = t.trim();
        let lower = t.to_lowercase();
        let label = ["e-mail:", "email:", "e-mail :", "email :"]
          .iter()
          .find(|p| lower.starts_with(*p))
          .map_or(0, |p| p.len());
        norm(if r == "email" { &t[label..] } else { t })
      };
      let pick = |c: &TCreator| {
        multiset(
          c.contacts
            .iter()
            .filter(|(r, t)| (r == "email") == emails && !norm(t).is_empty())
            .map(|(r, t)| format!("{r}\t{}", address(r, t))),
        )
      };
      let (sm, xm) = (pick(sc), pick(xc));
      // (an e-mail address the call gave as a url is an e-mail: the r2 brief's item 8, a url shaped like an e-mail is an
      // e-mail — today's `role=url` contact, 1701.01829, 2501.18366, 2506.18574)
      let as_url = |c: &TCreator, text: &str| {
        c.contacts
          .iter()
          .any(|(r, t)| r == "url" && norm(t) == text)
      };
      for item in minus(&sm, &xm) {
        let rule = sc.rules.get(&item).cloned();
        let rule = match item.split_once('\t') {
          Some(("email", text)) if as_url(xc, text) => {
            Some(format!("{} role:url-as-email", rule.unwrap_or_default()))
          },
          _ => rule,
        };
        push(kind, rule, format!("{}: shadow-only {item}", sc.name));
      }
      for item in minus(&xm, &sm) {
        let (role, text) = item.split_once('\t').unwrap_or(("", &item));
        let raw = xc
          .contacts
          .iter()
          .find(|(r, t)| r == role && address(r, t) == text)
          .map(|(_, t)| t.as_str());
        let rule = match (role, rule_of(shadow, role, text)) {
          ("url", None) => rule_of(shadow, "email", text).map(|r| format!("{r} role:url-as-email")),
          ("email", None) => raw.and_then(|raw| written_out_rule(shadow, raw)),
          (_, rule) => rule,
        };
        push(kind, rule, format!("{}: xml-only {item}", sc.name));
      }
    }
    // (a note's ORCID link is the creator's ORCID: today's note text has it moved out, 2106.12436)
    let orcids: Vec<String> = sc
      .contacts
      .iter()
      .chain(&xc.contacts)
      .filter(|(r, _)| r == "orcid")
      .map(|(_, t)| norm(t))
      .filter(|t| !t.is_empty())
      .collect();
    // (and an ORCID link a note keeps in its text, no one's contact — decision 2 of 2026-10-22: the XML's note prints
    // its logo, the shadow's its iD — `S. Donahue \orcidlink{…} is with …`, 2602.13176)
    let without_orcids = |n: &String| {
      let mut n = norm(n);
      for id in &orcids {
        n = n.replace(id.as_str(), "");
      }
      without_orcid_ids(&n)
    };
    // (the XML's notes of the creator: its note contacts and notes, and the footnotes set in its personname — the
    // text of a `\footnotemark` the name shows, `relocate_footnote`'s: `Steven Delvaux\footnotemark[1]` and the body's
    // `\footnotetext[1]{…}`, 1108.5155, 1404.2124; revtex's `\footnotetext[1]{E-mail: …}`, 2503.10108 — each compared:
    // one the shadow lacks is a disagreement, review revfms2a2_4 finding 1)
    let snotes = multiset(sc.notes.iter().map(without_orcids));
    let xnotes = multiset(xc.notes.iter().map(without_orcids));
    let in_name = multiset(xc.name_notes.iter().map(without_orcids));
    let unmatched = multiset(minus(&snotes, &xnotes).into_iter());
    for text in minus(&unmatched, &in_name) {
      push(
        "note",
        sc.rules.get(&format!("note\t{text}")).cloned(),
        format!("{}: shadow-only {text}", sc.name),
      );
    }
    for text in minus(&xnotes, &snotes) {
      push(
        "note",
        note_rule_of(shadow, &text),
        format!("{}: xml-only {text}", sc.name),
      );
    }
    for text in minus(&in_name, &unmatched) {
      push(
        "note",
        note_rule_of(shadow, &text),
        format!("{}: xml-only {text} (in the name)", sc.name),
      );
    }
  }
  // the block's own (the XML's nameless creator, OD #159), compared as a multiset as the creators' items are
  let block = |t: &Tuples| {
    multiset(
      t.block
        .iter()
        .filter(|(_, x)| !norm(x).is_empty())
        .map(|(r, x)| format!("{r}\t{}", norm(x))),
    )
  };
  let (sb, xb) = (block(shadow), block(xml));
  for item in minus(&sb, &xb) {
    push(
      "block",
      shadow.block_rules.get(&item).cloned(),
      format!("block: shadow-only {item}"),
    );
  }
  for item in minus(&xb, &sb) {
    let (role, text) = item.split_once('\t').unwrap_or(("", &item));
    let rule = if role == "note" {
      note_rule_of(shadow, text)
    } else {
      rule_of(shadow, role, text)
    };
    let rule = rule.or_else(|| {
      xml
        .block
        .iter()
        .find(|(r, x)| r == role && r == "email" && norm(x) == text)
        .and_then(|(_, x)| written_out_rule(shadow, x))
    });
    push("block", rule, format!("block: xml-only {item}"));
  }
  out
    .into_iter()
    .map(|(kind, rule, detail)| {
      let rule = rule.unwrap_or_else(|| "none".to_string());
      let class = if is_change_rule(&rule) {
        None
      } else {
        xml_wrong_class(&Context {
          kind,
          shadow,
          xml,
          detail: &detail,
          rule: &rule,
        })
      };
      Disagreement {
        kind,
        classified: class.is_some() || is_change_rule(&rule),
        class,
        rule,
        detail,
      }
    })
    .collect()
}

/// The matched `(shadow, xml)` index pairs of two name lists, in order.
pub(super) fn align(a: &[String], b: &[String]) -> Vec<(usize, usize)> {
  let (n, m) = (a.len(), b.len());
  if n.saturating_mul(m) > 4_000_000 {
    return (0..n.min(m))
      .filter(|&i| a[i] == b[i])
      .map(|i| (i, i))
      .collect();
  }
  let mut t = vec![0u32; (n + 1) * (m + 1)];
  let at = |i: usize, j: usize| i * (m + 1) + j;
  for i in (0..n).rev() {
    for j in (0..m).rev() {
      t[at(i, j)] = if a[i] == b[j] {
        t[at(i + 1, j + 1)] + 1
      } else {
        t[at(i + 1, j)].max(t[at(i, j + 1)])
      };
    }
  }
  let (mut i, mut j, mut pairs) = (0, 0, Vec::new());
  while i < n && j < m {
    if a[i] == b[j] {
      pairs.push((i, j));
      i += 1;
      j += 1;
    } else if t[at(i + 1, j)] >= t[at(i, j + 1)] {
      i += 1;
    } else {
      j += 1;
    }
  }
  pairs
}
