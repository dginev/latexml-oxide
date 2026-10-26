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

// The rule tags whose purpose is a change of today's output, and the test of a rule for one: the engine's, which the
// observer's writer reads too (FM S2b: a difference is written only when one of them decided it).
pub(crate) use latexml_engine::base_utilities::fm_is_change_rule as is_change_rule;

/// Whether a note's rule placed it by its order: a legend ruling 1 gives by its label (`ruling1:request`) or a note read
/// as a legend (`note:legend`) — today's relocation puts such a note last (review revfms2a2_9 finding 3).
fn places_by_order(rule: &str) -> bool {
  rule
    .split_whitespace()
    .any(|tag| matches!(tag, "ruling1:request" | "note:legend"))
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
  pieces_rule(shadow, &[(kind, &|_| true)], text)
}

/// A kind of shadow item and the rules of it a lookup takes ([`pieces_rule`]).
pub(super) type PieceKind<'a> = (&'a str, &'a dyn Fn(&str) -> bool);

/// [`rule_of`]'s sequences and parts: `text` as two or more of the shadow's items of the `kinds` (each with the rules it
/// takes) written together, else one line or a run of lines of an item its continuation lines joined.
pub(super) fn pieces_rule(shadow: &Tuples, kinds: &[PieceKind<'_>], text: &str) -> Option<String> {
  let pieces: Vec<(&str, &str)> = kinds
    .iter()
    .flat_map(|(kind, takes)| {
      let prefix = format!("{kind}\t");
      rule_maps(shadow)
        .flat_map(|rules| {
          rules.iter().filter_map(|(k, rule)| {
            k.strip_prefix(prefix.as_str())
              .filter(|p| !p.is_empty() && *p != text && takes(rule))
              .map(|p| (p, rule.as_str()))
          })
        })
        .collect::<Vec<_>>()
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
    // lexer reads `$\diamond$` as a mark, 0907.1243, FM S2a.2 r7 — the joined item's rule, `part …`: its lines, one or
    // a run of them, as the dump holds them, `continued`; FM S2a.2 r8, review revfms2a2_7 finding 5 — the middle of
    // a line explained nothing)
    _ => pieces
      .iter()
      .filter(|(piece, rule)| {
        rule.contains("line:continued")
          && text.len() >= 8
          && piece.len() > text.len()
          && piece.contains(text)
          && is_line_run(shadow, piece, text)
      })
      .map(|(_, rule)| format!("part {rule}"))
      .next(),
  }
}

/// Whether `text` is one line, or a run of lines, of a shadow entry its continuation lines joined into `piece`
/// ([`Tuples::continued`]); any part of it when the dump has no lines (one made before FM S2a.2 r8: `None` — an r8
/// dump's list may be empty, review revfms2a2_8 finding 4).
pub(super) fn is_line_run(shadow: &Tuples, piece: &str, text: &str) -> bool {
  let Some(continued) = &shadow.continued else {
    return true;
  };
  continued
    .iter()
    .filter(|lines| lines.concat() == piece)
    .any(|lines| {
      (0..lines.len()).any(|i| (i + 1..=lines.len()).any(|j| lines[i..j].concat() == text))
    })
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
/// An API entry's too, led by its symbol (`api:leading-mark`: `\affiliation{… \\ $^\star${corresponding author, e-mail:
/// …}}`, 0812.1531 — FM S2a.2 r8, review revfms2a2_7 finding 8).
///
/// Or the shadow's affiliations and an API entry's legend notes written together: today's one contact of a call's
/// affiliations and its legend (`\country{$^1$SJTU, $^2$Infinigence-AI, $^*$Equal contributions}`, 2409.11440; elsarticle's
/// `\address{$^1$… \\ $^*$Corresponding author …}`, 2306.16240) — the first piece's rule (`overlaps …`); a block's
/// legend lines welded on are the PDF-confirmed class (`legends_welded_on`).
///
/// An API entry's legend is one by its own tag, `api:legend` (the role change, review revfms2a2_8 finding 2, FM S2a.2 r9),
/// not by the leading mark that labels it.
pub(super) fn affiliation_rule_of(shadow: &Tuples, text: &str) -> Option<String> {
  let legend = |rule: &str| {
    rule.contains("ruling1:request")
      || rule.contains("ruling2:symbol-line")
      || rule.contains("api:legend")
  };
  rule_of(shadow, "affiliation", text)
    .or_else(|| rule_of(shadow, "note", text).filter(|rule| legend(rule)))
    .or_else(|| {
      pieces_rule(
        shadow,
        &[
          ("affiliation", &|_| true),
          ("note", &|rule| rule.contains("api:legend")),
        ],
        text,
      )
    })
}

/// The shadow-only items of a side (`items`, keyed by `key`), each with its rule: when the shadow holds several copies of
/// one key (`all`, every copy's rule in order), the copies the XML holds take the rules that reproduce today's (no
/// change tag) first and the copies it lacks the rest, one each — never one rule standing for every copy (review
/// revfms2a2_8 finding 3b: 1910.01401's second ORCID copy, placed by `owner:adjacent`, read as the ruling's
/// `note:adjacent ruling4:inner-contact`); a key with no rules list its one rule (`rules`).
pub(super) fn surplus_rules(
  items: Vec<String>,
  key: impl Fn(&str) -> String,
  rules: &BTreeMap<String, String>,
  all: &BTreeMap<String, Vec<String>>,
) -> Vec<(String, Option<String>)> {
  let mut surplus: BTreeMap<String, usize> = BTreeMap::new();
  for item in &items {
    *surplus.entry(key(item)).or_default() += 1;
  }
  let mut taken: BTreeMap<String, usize> = BTreeMap::new();
  items
    .into_iter()
    .map(|item| {
      let k = key(&item);
      let n = taken.entry(k.clone()).or_default();
      let at = *n;
      *n += 1;
      let count = surplus[&k];
      let rule = match all.get(&k) {
        Some(copies) if copies.len() >= count && copies.len() > 1 => {
          let mut ordered: Vec<&String> = copies.iter().filter(|r| !is_change_rule(r)).collect();
          ordered.extend(copies.iter().filter(|r| is_change_rule(r)));
          ordered.get(copies.len() - count + at).map(|r| (*r).clone())
        },
        _ => rules.get(&k).cloned(),
      };
      (item, rule)
    })
    .collect()
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
    let affiliation_key = |text: &str| format!("affiliation\t{text}");
    for (text, rule) in surplus_rules(minus(&sa, &xa), affiliation_key, &sc.rules, &sc.all_rules) {
      push(
        "affiliation",
        rule,
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
      for (item, rule) in surplus_rules(minus(&sm, &xm), str::to_string, &sc.rules, &sc.all_rules) {
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
    let note_key = |text: &str| format!("note\t{text}");
    for (text, rule) in surplus_rules(
      minus(&unmatched, &in_name),
      note_key,
      &sc.rules,
      &sc.all_rules,
    ) {
      push("note", rule, format!("{}: shadow-only {text}", sc.name));
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
    // (the notes both hold, in the order each prints them: a note the shadow placed late came after the ones the source
    // gives later — review revfms2a2_7 finding 1, FM S2a.2 r8. The notes a ruling placed by their order (a legend ruling 1
    // places by its label, `ruling1:request`, or a note read as a legend, `note:legend`: today's relocates it last, where
    // the shadow keeps the source's order — `$\dagger$Corresponding author` before the funding note, 2607.03557) taken
    // out of both, when the XML has them last, the rest must agree in order: then the first placed note's rule explains
    // it; else the first of the rest out of today's order, a rule of no change — review revfms2a2_8 finding 3a, FM S2a.2
    // r9: `[L, A, B]` against `[B, A, L]` was the legend's. A tag that says whose note it is (`owner:spelled`,
    // `ruling4:inner-contact`) says nothing of where it sits: no excuse — review revfms2a2_9 finding 3, FM S2b r1)
    if let Some((shadow_order, xml_order)) = note_orders(&snotes, &xnotes, sc, xc, &without_orcids)
    {
      let note_rule = |key: &String| sc.rules.get(&format!("note\t{key}")).cloned();
      let by_order = |key: &&String| note_rule(key).is_some_and(|rule| places_by_order(&rule));
      // (today's relocation puts the placed notes after the rest)
      let last_in_xml = xml_order
        .iter()
        .position(|k| by_order(&k))
        .is_none_or(|at| xml_order[at..].iter().all(|k| by_order(&k)));
      let placed = |key: &&String| last_in_xml && by_order(key);
      let shadow_rest: Vec<&String> = shadow_order.iter().filter(|k| !placed(k)).collect();
      let xml_rest: Vec<&String> = xml_order.iter().filter(|k| !placed(k)).collect();
      let rule = if shadow_rest == xml_rest {
        shadow_order
          .iter()
          .find(|k| placed(k))
          .and_then(note_rule)
          .map(|rule| format!("order {rule}"))
      } else {
        // (the first of the rest out of today's order: whatever its tags say of whose note it is, none placed it there —
        // its rule written as one word, `order-of:<tag>+<tag>`, no change tag)
        shadow_rest
          .iter()
          .zip(&xml_rest)
          .find(|(a, b)| a != b)
          .and_then(|(a, _)| note_rule(a))
          .map(|rule| {
            format!(
              "order-of:{}",
              rule.split_whitespace().collect::<Vec<_>>().join("+")
            )
          })
      };
      push(
        "note-order",
        rule,
        format!(
          "{}: shadow [{}] xml [{}]",
          sc.name,
          shadow_order.join(" | "),
          xml_order.join(" | ")
        ),
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
  for (item, rule) in surplus_rules(
    minus(&sb, &xb),
    str::to_string,
    &shadow.block_rules,
    &shadow.block_all_rules,
  ) {
    push("block", rule, format!("block: shadow-only {item}"));
  }
  for item in minus(&xb, &sb) {
    let (role, text) = item.split_once('\t').unwrap_or(("", &item));
    let rule = match role {
      "note" => note_rule_of(shadow, text),
      "affiliation" => affiliation_rule_of(shadow, text),
      _ => rule_of(shadow, role, text),
    };
    let rule = rule.or_else(|| {
      xml
        .block
        .iter()
        .find(|(r, x)| r == role && r == "email" && norm(x) == text)
        .and_then(|(_, x)| written_out_rule(shadow, x))
    });
    // (or a note the shadow gave the title, which today's left in a nameless creator under the role its mark's entry
    // took — `\altaffilmark` in `\title`, its `\altaffiltext` an altaffiliation, 1801.03570)
    let rule = rule.or_else(|| {
      shadow
        .block_rules
        .get(&format!("note\t{text}"))
        .filter(|r| r.contains("api:title-note"))
        .map(|r| format!("moved {r}"))
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

/// The notes a creator's two sides both hold (`shadow` and `xml`, as multisets), each side's in its order, when the
/// orders differ.
pub(super) fn note_orders(
  shadow: &BTreeMap<String, usize>,
  xml: &BTreeMap<String, usize>,
  sc: &TCreator,
  xc: &TCreator,
  key: &dyn Fn(&String) -> String,
) -> Option<(Vec<String>, Vec<String>)> {
  let common: BTreeMap<String, usize> = shadow
    .iter()
    .filter_map(|(k, n)| xml.get(k).map(|m| (k.clone(), (*n).min(*m))))
    .collect();
  let order = |notes: &[String]| {
    let mut left = common.clone();
    notes
      .iter()
      .map(key)
      .filter(|k| {
        left.get_mut(k).is_some_and(|n| {
          let take = *n > 0;
          *n = n.saturating_sub(1);
          take
        })
      })
      .collect::<Vec<_>>()
  };
  let (s, x) = (order(&sc.notes), order(&xc.notes));
  (s != x).then_some((s, x))
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
