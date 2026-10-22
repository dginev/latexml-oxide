//! `ab_audit fmshadow ROOT`: the frontmatter observer's shadow resolution (`<dest>.fmshadow.json`, written under
//! `LATEXML_FM_SHADOW=1`; OXIDIZED_DESIGN_DIVERGENCES #837) against the frontmatter of the XML beside it, per paper.
//!
//! Both sides are read into the same tuples: per creator its name (marks stripped: an `ltx:sup` in the personname is
//! a mark), its affiliations (`affiliation`/`address` contacts), its other contacts by role, its notes; and the
//! author block's own (the XML's trailing nameless creator, OD #159), compared as a multiset too. Each disagreement is
//! classified by its kind (`name-split`, `affiliation`, `contact-owner`, `contact`, `note`, `block`), the family (the
//! policy key's binding) and the shadow rule that decided it; a disagreement is **classified** when that rule holds an
//! adopted ruling's change tag ([`CHANGE_TAGS`]: the output S2b is meant to change) or it falls in a PDF-confirmed
//! XML-wrong class ([`XML_WRONG`], listed by name with its witnesses), and **unclassified** otherwise: a place the
//! shadow was meant to reproduce today's output and does not (FM S2a.2 r2, review revfms2a2 findings 8, 9).
//! A paper whose ledger has no rehearsed block is "no block" (a binding's single-author calls: outside the gate's
//! denominator, notes decision 1); one whose rehearsal held diagnostics is "held" (notes decision 3).
//!
//! Output: `<prefix>.tsv` (a row per paper), `<prefix>.disagree.tsv` (a row per disagreement), and
//! `<prefix>.summary.txt` (the per-family table ranked by reach, with the gate number: the share of the family's
//! counted papers with an unclassified disagreement).

use std::{
  collections::BTreeMap,
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use latexml_engine::base_utilities::{
  FmAssoc, FmEvent, FmEventKind, FmInput, FmResolved, FmSepKind, fm_creator_call, fm_resolve,
};
use roxmltree::Node;
use serde_json::Value;

use crate::{
  facts::{collapse, is_ltx, parse},
  tree::{discover, par_map},
};

/// One side's tuples.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Tuples {
  pub creators:    Vec<TCreator>,
  /// the author block's own contacts and notes, `(role, text)` — role `affiliation` for an affiliation or address,
  /// `note` for a note ([`block_role`])
  pub block:       Vec<(String, String)>,
  /// the shadow's rules for the block's items: `role\ttext` → rule (the XML's empty)
  pub block_rules: BTreeMap<String, String>,
  /// the shadow's rules for what the rulings dropped (an emptied note, ruling 4): `note\ttext` → rule
  pub dropped:     BTreeMap<String, String>,
}

/// The role a block item is compared under: an affiliation or an address `affiliation`, a note of any role `note`, a
/// contact its role.
fn block_role(role: &str) -> String {
  match role {
    "affiliation" | "address" => "affiliation".into(),
    "note" | "thanks" | "legend" | "footnotetext" => "note".into(),
    other => other.to_string(),
  }
}

/// One creator's tuples.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TCreator {
  pub name:         String,
  pub affiliations: Vec<String>,
  /// `(role, text)`
  pub contacts:     Vec<(String, String)>,
  pub notes:        Vec<String>,
  /// the shadow's rules, by item: `name` the split rule; else `role\ttext` → rule
  pub rules:        BTreeMap<String, String>,
  /// the shadow's marks shown on the name (its requests, prefixes aside), normalised
  pub marks:        Vec<String>,
}

/// Text compared by its letters and digits alone, lowercased (the XML's markup and spacing, the ink's punctuation
/// aside).
pub fn norm(text: &str) -> String {
  text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

/// The text a formula in a name prints when it is a mark: none for a floating superscript (`{}^{a,\;b}`, `^{1}`:
/// revtex names written `Dadhich${}^{a,\;b}$`, 1307.6166); the name for `\text{Leon Bello}^{1,3}` (2011.08099); `None`
/// for any other formula (its text kept).
fn math_mark_text(math: Node<'_, '_>) -> Option<String> {
  let mut tex = math.attribute("tex")?.trim();
  // (spacing before the script: `\,^{3}`, 0804.4669)
  while let Some(rest) = [r"\,", r"\;", r"\!", r"\ ", "~", "{}"]
    .iter()
    .find_map(|p| tex.strip_prefix(p))
  {
    tex = rest.trim_start();
  }
  if tex.starts_with('^') {
    return Some(String::new());
  }
  for command in [r"\text{", r"\textrm{", r"\mbox{", r"\mathrm{"] {
    if let Some(rest) = tex.strip_prefix(command)
      && let Some(close) = rest.find('}')
      && rest[close + 1..].trim_start().starts_with('^')
    {
      return Some(rest[..close].to_string());
    }
  }
  None
}

/// The text of `node` without its `ltx:sup` descendants (a name's or an affiliation's marks) nor its notes, a formula
/// that is a mark as [`math_mark_text`] reads it.
fn text_without_marks(node: Node<'_, '_>) -> String {
  fn walk(node: Node<'_, '_>, out: &mut String) {
    for child in node.children() {
      if child.is_text() {
        out.push_str(child.text().unwrap_or(""));
      } else if is_ltx(child, "Math")
        && let Some(text) = math_mark_text(child)
      {
        out.push_str(&text);
      } else if child.is_element() && !is_ltx(child, "sup") && !is_ltx(child, "note") {
        walk(child, out);
      }
      if is_ltx(child, "break") {
        out.push(' ');
      }
    }
  }
  let mut raw = String::new();
  walk(node, &mut raw);
  collapse(&raw)
}

/// The XML's tuples.
pub fn xml_tuples(xml: &[u8]) -> Result<Tuples, String> {
  let doc = parse(xml)?;
  let mut out = Tuples::default();
  for node in doc
    .root_element()
    .descendants()
    .filter(|n| is_ltx(*n, "creator"))
  {
    let mut creator = TCreator::default();
    let mut names = Vec::new();
    for child in node.children() {
      if is_ltx(child, "personname") {
        names.push(text_without_marks(child));
        // (an ORCID link printed in the name, `\orcidlink` beside it: the creator's ORCID, as the shadow's contact)
        for link in child.descendants().filter(|n| is_ltx(*n, "ref")) {
          if let Some(href) = link.attribute("href").filter(|h| h.contains("orcid.org/")) {
            creator
              .contacts
              .push(("orcid".into(), orcid_id(href).to_string()));
          }
        }
      } else if is_ltx(child, "contact") {
        let role = child.attribute("role").unwrap_or("").to_string();
        let mut text = text_without_marks(child);
        // (an ORCID contact prints a logo, its identifier the link's: `<ref href="https://orcid.org/0000-…">`)
        if text.is_empty()
          && let Some(href) = child
            .descendants()
            .find(|n| is_ltx(*n, "ref"))
            .and_then(|r| r.attribute("href"))
        {
          text = orcid_id(href).to_string();
        }
        if role == "affiliation" || role == "address" {
          creator.affiliations.push(text);
        } else if is_note_role(&role) {
          creator.notes.push(text);
        } else {
          creator.contacts.push((role, text));
        }
      } else if is_ltx(child, "note") {
        creator.notes.push(text_without_marks(child));
      }
    }
    creator.name = names.join(" ");
    if creator.name.trim().is_empty() {
      out.block.extend(
        creator
          .affiliations
          .into_iter()
          .map(|a| ("affiliation".to_string(), a)),
      );
      out.block.extend(
        creator
          .contacts
          .into_iter()
          .map(|(r, t)| (block_role(&r), t)),
      );
      out
        .block
        .extend(creator.notes.into_iter().map(|n| ("note".to_string(), n)));
    } else {
      out.creators.push(creator);
    }
  }
  Ok(out)
}

/// An ORCID link's identifier (`https://orcid.org/0000-0002-…` "0000-0002-…"), any other text as it is.
fn orcid_id(text: &str) -> &str {
  let text = text.trim();
  ["https://orcid.org/", "http://orcid.org/", "orcid.org/"]
    .iter()
    .find_map(|p| text.strip_prefix(p))
    .unwrap_or(text)
}

/// A contact role that is a note in both tuples (revtex's `\thanks` a `role=note` contact, a `thanks` contact): the
/// comparison does not tell a note given as a contact from a note.
fn is_note_role(role: &str) -> bool { matches!(role, "note" | "thanks") }

fn assoc_rule(rules: &mut BTreeMap<String, String>, kind: &str, assoc: &FmAssoc) {
  let text = if kind == "orcid" {
    orcid_id(&assoc.text)
  } else {
    &assoc.text
  };
  rules.insert(format!("{kind}\t{}", norm(text)), assoc.rule.clone());
}

/// The shadow's tuples, from a resolution.
pub fn shadow_tuples(resolved: &FmResolved) -> Tuples {
  let mut out = Tuples::default();
  for c in &resolved.creators {
    if c.name.trim().is_empty() {
      continue;
    }
    let mut creator = TCreator {
      name: c.name.clone(),
      ..TCreator::default()
    };
    creator.rules.insert("name".into(), c.rule.clone());
    creator.marks = c
      .requests
      .iter()
      .map(|r| norm(r.rsplit_once(':').map_or(r.as_str(), |(_, l)| l)))
      .filter(|m| !m.is_empty())
      .collect();
    for a in &c.affiliations {
      creator.affiliations.push(a.text.clone());
      assoc_rule(&mut creator.rules, "affiliation", a);
    }
    for a in &c.contacts {
      if is_note_role(&a.role) {
        creator.notes.push(a.text.clone());
        assoc_rule(&mut creator.rules, "note", a);
      } else {
        let text = if a.role == "orcid" {
          orcid_id(&a.text).to_string()
        } else {
          a.text.clone()
        };
        creator.contacts.push((a.role.clone(), text));
        assoc_rule(&mut creator.rules, &a.role, a);
      }
    }
    for a in &c.notes {
      creator.notes.push(a.text.clone());
      assoc_rule(&mut creator.rules, "note", a);
    }
    out.creators.push(creator);
  }
  for a in &resolved.dropped {
    out
      .dropped
      .insert(format!("note\t{}", norm(&a.text)), a.rule.clone());
  }
  for a in &resolved.block {
    let role = block_role(&a.role);
    out
      .block_rules
      .insert(format!("{role}\t{}", norm(&a.text)), a.rule.clone());
    out.block.push((role, a.text.clone()));
  }
  out
}

// ---- the dump's JSON back into the resolver's inputs (`--re-resolve`) and its tuples ----

fn s(v: &Value) -> String { v.as_str().unwrap_or("").to_string() }

/// A dump event as the lexer's event.
pub fn json_event(v: &Value) -> FmEvent {
  let kind = match v["kind"].as_str().unwrap_or("") {
    "Name" => FmEventKind::Name,
    "Mark" => FmEventKind::Mark,
    "Break" => FmEventKind::Break,
    "Contact" => FmEventKind::Contact,
    "Note" => FmEventKind::Note,
    "ApiCall" => FmEventKind::ApiCall,
    "Sep" => FmEventKind::Sep(match v["sep"].as_str().unwrap_or("") {
      "and" => FmSepKind::And(match v["command"].as_str() {
        Some("\\And") => "\\And",
        Some("\\AND") => "\\AND",
        _ => "\\and",
      }),
      "column" => FmSepKind::Column("&"),
      "comma" => FmSepKind::Comma,
      "word" => FmSepKind::Word(s(&v["word"])),
      _ => FmSepKind::Gap {
        items: v["items"].as_u64().unwrap_or(0) as usize,
        width: v["width"].as_f64().unwrap_or(0.0),
        fill:  v["fill"].as_bool().unwrap_or(false),
      },
    }),
    _ => FmEventKind::Layout,
  };
  let mut event = FmEvent::new(kind);
  event.span = v["span"].as_array().and_then(|a| {
    Some((
      u16::try_from(a.first()?.as_u64()?).ok()?,
      u16::try_from(a.get(1)?.as_u64()?).ok()?,
    ))
  });
  event.text = s(&v["text"]);
  event.label = v["label"].as_str().map(str::to_string);
  event.symbol = v["symbol"].as_bool().unwrap_or(false);
  event.line_first = v["line_first"].as_bool().unwrap_or(false);
  event.role = v["role"].as_str().map(str::to_string);
  event.target = v["target"].as_str().map(str::to_string);
  event.nest = v["nest"].as_u64().unwrap_or(0) as u8;
  event.inner = v["inner"]
    .as_array()
    .map(|a| a.iter().map(json_event).collect())
    .unwrap_or_default();
  event
}

/// The resolver's inputs from a dump (as `resolver::ledger_inputs` makes them from the ledger).
pub fn json_inputs(dump: &Value) -> Vec<FmInput> {
  let mut inputs = Vec::new();
  for e in dump["entries"].as_array().into_iter().flatten() {
    match e["kind"].as_str() {
      Some("author") if e["status"] == "events" => inputs.push(FmInput::Block {
        index:   e["index"].as_u64().unwrap_or(0) as usize,
        binding: s(&e["binding"]),
        hook:    s(&e["hook"]),
        keyvals: e["keyvals"].as_str().map(str::to_string),
        counter: e["counter"].as_i64(),
        replace: e["replace"].as_bool().unwrap_or(false),
        epoch:   e["epoch"].as_u64().unwrap_or(0) as usize,
        events:  e["events"]
          .as_array()
          .into_iter()
          .flatten()
          .filter(|ev| ev["kind"] != "ApiCall")
          .map(json_event)
          .collect(),
      }),
      Some("api")
        if e["inside"] == false
          && fm_creator_call(
            e["tag"].as_str().unwrap_or(""),
            e["parent"].as_str().unwrap_or(""),
          ) =>
      {
        inputs.push(FmInput::Api {
          index:  e["index"].as_u64().unwrap_or(0) as usize,
          tag:    s(&e["tag"]),
          role:   s(&e["role"]),
          attrs:  s(&e["attrs"]),
          events: e["events"]
            .as_array()
            .into_iter()
            .flatten()
            .map(json_event)
            .collect(),
        })
      },
      _ => {},
    }
  }
  inputs
}

/// A dump's `resolved` section as the resolver's value.
pub fn json_resolved(v: &Value) -> FmResolved {
  let assoc = |a: &Value| FmAssoc {
    role:   s(&a["role"]),
    text:   s(&a["text"]),
    label:  a["label"].as_str().map(str::to_string),
    rule:   s(&a["rule"]),
    source: None,
  };
  let assocs = |a: &Value| {
    a.as_array()
      .into_iter()
      .flatten()
      .map(assoc)
      .collect::<Vec<_>>()
  };
  let mut out = FmResolved::default();
  for c in v["creators"].as_array().into_iter().flatten() {
    out
      .creators
      .push(latexml_engine::base_utilities::FmCreator {
        name:         s(&c["name"]),
        requests:     c["requests"]
          .as_array()
          .into_iter()
          .flatten()
          .map(s)
          .collect(),
        rule:         s(&c["rule"]),
        source:       c["source"].as_u64().unwrap_or(0) as usize,
        affiliations: assocs(&c["affiliations"]),
        contacts:     assocs(&c["contacts"]),
        notes:        assocs(&c["notes"]),
      });
  }
  out.block = assocs(&v["block"]);
  out.dropped = assocs(&v["dropped"]);
  out
}

// ---- the comparison ----

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
///   events: by construction only a real Mark, `resolver.rs` `api`), the `\ref`-mark keys (`api:ref-label`, `ref:`);
/// - ruling 2, a footnote-symbol line labels the line for its carriers: `ruling2:symbol-line`, `ruling2:uncarried`;
/// - the corresponding author (charter §4's e-mail owner order): `owner:corresponding`;
/// - one contact per print event (`ruling3:one-per-print`): counted only where the XML holds the pieces joined into one
///   item (an `overlaps` rule), the change the ruling makes — the owner of each address is its other tag's;
/// - an emptied note dropped, a note's inner contacts its contacts, `\thanks{}` kept: `ruling4:`;
/// - a url whose text is one e-mail address is an e-mail (`role:url-as-email`: the S2a.2 r2 brief's item 8, adopted);
/// - a list with one address per author that every spelled address agrees with is in author order (`owner:order`:
///   charter §4's owner order, 2026-10-21, OD #805 — for every class, not only the sequenced lists).
///
/// Elimination (`owner:elimination`, the one address left to the one name left) is NOT a change: today's output
/// applies it too (OXIDIZED_DESIGN_DIVERGENCES #52's e-mail lines, `owners_by_elimination`, under user ruling R10
/// 2026-10-20, 77p r10;
/// FIDELITY_CHARTER §4's owner order is silent on it), so its disagreements are the shadow's to explain.
const CHANGE_TAGS: &[&str] = &[
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

/// A class of disagreement where today's XML is wrong and the shadow right, confirmed against the PDFs of its witnesses
/// (FM S2a.2 r2: a disagreement is classified only by an adopted ruling's change, [`CHANGE_TAGS`], or by one of these,
/// listed by name).
pub struct XmlWrong {
  pub name:      &'static str,
  /// the arXiv papers whose PDF confirmed it
  pub witnesses: &'static [&'static str],
  pub applies:   fn(&Context<'_>) -> bool,
}

/// How an XML name list can differ from the shadow's in an [`XML_WRONG`] name class.
#[derive(Clone, Copy)]
enum NameFault {
  /// two or more of the shadow's names, consecutive, joined into one XML name, each after the first split out by a
  /// rule `parted` accepts (given that rule and the name before it)
  Merged(fn(&str, &TCreator) -> bool),
  /// a shadow name with the letters "ID" after it (an ORCID logo's glyphs read as text)
  OrcidLetters,
  /// the e-mail addresses the names line prints read as XML creators of their own, or into the name before them
  Addresses,
  /// a shadow name with its own affiliations after it in one XML name
  WithAffiliation,
}

/// Whether a text is e-mail addresses alone (brackets, braces and separators aside).
fn is_addresses(text: &str) -> bool {
  let words: Vec<&str> = text
    .split(|c: char| c.is_whitespace() || ",;(){}<>[]".contains(c))
    .filter(|w| !w.is_empty())
    .collect();
  !words.is_empty()
    && words.iter().all(|w| {
      w.trim_matches(|c: char| ".:".contains(c))
        .split_once('@')
        .is_some_and(|(l, d)| !l.is_empty() && d.contains('.'))
    })
}

/// Whether every XML creator name is the shadow's at its place, or (at least once) the shadow's under `fault`: the
/// whole name disagreement is that class's.
fn names_explained(shadow: &Tuples, xml: &Tuples, fault: NameFault) -> bool {
  let sn: Vec<String> = shadow.creators.iter().map(|c| norm(&c.name)).collect();
  let xn: Vec<String> = xml_names(shadow, xml);
  let (mut i, mut j, mut used) = (0usize, 0usize, false);
  while j < xn.len() {
    if i < sn.len() && sn[i] == xn[j] {
      i += 1;
      j += 1;
      continue;
    }
    match fault {
      NameFault::OrcidLetters => {
        // (or the identifier as text: LMCS's `ORCID:0000-0003-2914-2734` after the name, where the PDF shows the logo,
        // 2209.02767, 1902.02698)
        let as_text = |x: &str, n: &str| {
          x.strip_prefix(n)
            .and_then(|rest| rest.strip_prefix("orcid"))
            .is_some_and(|id| id.len() >= 15 && id.chars().all(|c| c.is_ascii_digit() || c == 'x'))
        };
        if i < sn.len() && (xn[j] == format!("{}id", sn[i]) || as_text(&xn[j], &sn[i])) {
          (i, j, used) = (i + 1, j + 1, true);
          continue;
        }
        return false;
      },
      NameFault::WithAffiliation => {
        let affiliations: String = shadow
          .creators
          .get(i)
          .map(|c| c.affiliations.iter().map(|a| norm(a)).collect())
          .unwrap_or_default();
        if i < sn.len() && !affiliations.is_empty() && xn[j] == format!("{}{affiliations}", sn[i]) {
          (i, j, used) = (i + 1, j + 1, true);
          continue;
        }
        return false;
      },
      NameFault::Addresses => {
        let raw = &xml.creators[j].name;
        if is_addresses(raw) {
          (j, used) = (j + 1, true);
          continue;
        }
        // (the name, then its addresses: `Ann Able (ann@x.org)`)
        if i < sn.len()
          && let Some(at) = raw.find('@')
          && let Some(start) = raw[..at].rfind(|c: char| c.is_whitespace() || "({<".contains(c))
          && norm(&raw[..start]) == sn[i]
          && is_addresses(&raw[start..])
        {
          (i, j, used) = (i + 1, j + 1, true);
          continue;
        }
        return false;
      },
      NameFault::Merged(parted) => {
        // (the XML's one name holds the pieces in order, the word "and" between two of them allowed: `A, B, C and
        // D`, 0903.4816)
        let x = xn[j].as_str();
        let mut at = 0usize;
        let mut k = i;
        while k < sn.len() && at < x.len() {
          // (no address is a name, 2507.03885)
          if shadow.creators[k].name.contains('@') {
            return false;
          }
          if k > i
            && !parted(
              shadow.creators[k]
                .rules
                .get("name")
                .map_or("", String::as_str),
              &shadow.creators[k - 1],
            )
          {
            return false;
          }
          if x[at..].starts_with(sn[k].as_str()) {
            at += sn[k].len();
          } else if k > i && x[at..].starts_with("and") && x[at + 3..].starts_with(sn[k].as_str()) {
            at += 3 + sn[k].len();
          } else {
            return false;
          }
          k += 1;
        }
        // (each piece a name of two words or more, unless the run is a long list: `Sanhueza, Patricio` is one reversed
        // name, 2604.19097; `…, Q. An, Anita, Y. Bai, …` a collaboration's list, 2007.02542)
        let one_word = (i..k).any(|p| !shadow.creators[p].name.trim().contains(' '));
        if k - i < 2 || at != x.len() || (one_word && k - i < 5) {
          return false;
        }
        (i, j, used) = (k, j + 1, true);
      },
    }
  }
  used && i == sn.len()
}

/// Whether the XML's one creator holds the shadow's names, five or more, in order (the word "and" between two allowed)
/// and more after them: a collaboration's whole author block — its names, marks and affiliations — read into one
/// personname (BESIII, 2007.02542, 0710.0039; LIGO, 1310.0383).
fn block_in_one_name(shadow: &Tuples, xml: &Tuples) -> bool {
  let [only] = xml.creators.as_slice() else {
    return false;
  };
  if shadow.creators.len() < 5 {
    return false;
  }
  let x = norm(&only.name);
  let mut at = 0usize;
  for c in &shadow.creators {
    let n = norm(&c.name);
    if x[at..].starts_with(n.as_str()) {
      at += n.len();
    } else if x[at..].starts_with("and") && x[at + 3..].starts_with(n.as_str()) {
      at += 3 + n.len();
    } else {
      return false;
    }
  }
  true
}

/// The shadow's names the XML lost to an affiliation: the XML's names are the shadow's but these (in order), each lost
/// name's text inside an XML affiliation or address — a later names row read as the row above's affiliation
/// (`Eric S. Egge \\ … \\ Toufik Mansour \\ LaBRI …`, math0203226; 0908.1348, 2409.04981, 2509.10952, 2508.04012).
fn names_lost_to_affiliations(shadow: &Tuples, xml: &Tuples) -> Vec<String> {
  let sn: Vec<String> = shadow.creators.iter().map(|c| norm(&c.name)).collect();
  let xn = xml_names(shadow, xml);
  let pairs = align(&sn, &xn);
  if pairs.len() != xn.len() || pairs.len() == sn.len() {
    return Vec::new();
  }
  let held: Vec<String> = xml
    .creators
    .iter()
    .flat_map(|c| c.affiliations.iter().map(|a| norm(a)))
    .chain(xml.block.iter().map(|(_, t)| norm(t)))
    .collect();
  let lost: Vec<String> = (0..sn.len())
    .filter(|i| !pairs.iter().any(|(a, _)| a == i))
    .map(|i| sn[i].clone())
    .collect();
  if lost
    .iter()
    .all(|n| n.len() >= 6 && held.iter().any(|h| h.contains(n.as_str())))
  {
    lost
  } else {
    Vec::new()
  }
}

/// The PDF-confirmed XML-wrong classes (FM S2a.2 r2; each witness's pdflatex PDF read against both sides).
pub const XML_WRONG: &[XmlWrong] = &[
  XmlWrong {
    name:      "names parted by a wide space (\\quad, \\enspace, \\hspace, spaces) merged into one personname",
    witnesses: &["2312.06207", "2303.02369", "2512.16241", "2511.18287"],
    applies:   |c| {
      c.kind == "name-split"
        && names_explained(
          c.shadow,
          c.xml,
          NameFault::Merged(|rule, _| rule.contains("sep:gap")),
        )
    },
  },
  XmlWrong {
    name:      "a list of names (commas, \"and\") in one personname",
    witnesses: &["1512.06985", "1610.06365", "2211.02638", "2411.11150"],
    applies:   |c| {
      c.kind == "name-split"
        && names_explained(
          c.shadow,
          c.xml,
          // (a list's names parted by commas and "and", a wide space between two of them too: 2007.02542)
          NameFault::Merged(|rule, _| rule.contains("sep:")),
        )
    },
  },
  XmlWrong {
    name:      "a later names row read as the affiliation of the row above",
    witnesses: &[
      "math0203226",
      "0908.1348",
      "2409.04981",
      "2509.10952",
      "2508.04012",
      "2208.13126",
      "2210.14979",
    ],
    applies:   |c| {
      let lost = names_lost_to_affiliations(c.shadow, c.xml);
      !lost.is_empty()
        && (c.kind == "name-split"
          || c.detail.split_once("xml-only ").is_some_and(|(_, text)| {
            lost.iter().any(|n| {
              text
                .trim()
                .trim_start_matches("affiliation\t")
                .contains(n.as_str())
            })
          }))
    },
  },
  XmlWrong {
    name:      "a name with its affiliation in one personname",
    witnesses: &["1611.06737", "2601.04271", "1811.08870"],
    applies:   |c| {
      c.kind == "name-split" && names_explained(c.shadow, c.xml, NameFault::WithAffiliation)
    },
  },
  XmlWrong {
    name:      "a collaboration's whole author list read into one personname",
    witnesses: &["2007.02542", "0710.0039", "1310.0383"],
    applies:   |c| c.kind == "name-split" && block_in_one_name(c.shadow, c.xml),
  },
  XmlWrong {
    name:      "names parted only by a mark or a note merged into one personname",
    witnesses: &[
      "1609.02129",
      "1803.06822",
      "2511.04278",
      "2508.08510",
      "2312.03108",
      "1412.3142",
    ],
    applies:   |c| {
      c.kind == "name-split"
        && names_explained(
          c.shadow,
          c.xml,
          // (the name before showing a mark or a note: `C. Nisoli$^2$ C. Reichhardt`, `Sara Spangelo\thanks{…} Raj
          // Katti`, 1412.3142)
          NameFault::Merged(|rule, before| {
            (!before.marks.is_empty() || !before.notes.is_empty())
              && !rule.contains("sep:")
              && rule.starts_with("line:names")
          }),
        )
    },
  },
  XmlWrong {
    name:      "a note made in the title (before any author) given to the first author",
    witnesses: &["1603.01892", "1802.05590", "cond-mat0502298"],
    applies:   |c| matches!(c.kind, "note" | "block") && c.rule.contains("api:title-note"),
  },
  XmlWrong {
    name:      "e-mail addresses printed in the names line read as creators or into the name before them",
    witnesses: &["0810.4404", "1801.04987", "2405.06319", "2507.03885"],
    applies:   |c| {
      names_explained(c.shadow, c.xml, NameFault::Addresses)
        && (c.kind == "name-split"
          || c
            .detail
            .split_once("shadow-only email\t")
            .is_some_and(|(_, text)| {
              c.xml
                .creators
                .iter()
                .any(|x| norm(&x.name).contains(text.trim()))
            }))
    },
  },
  XmlWrong {
    name:      "an ORCID logo's glyphs read as the letters \"ID\" (or its identifier as text) after the name",
    witnesses: &["2501.15882", "2405.19400", "2209.02767", "1902.02698"],
    applies:   |c| {
      c.kind == "name-split" && names_explained(c.shadow, c.xml, NameFault::OrcidLetters)
    },
  },
];

/// The first [`XML_WRONG`] class a disagreement falls in.
fn xml_wrong_class(context: &Context<'_>) -> Option<&'static str> {
  XML_WRONG
    .iter()
    .find(|class| (class.applies)(context))
    .map(|class| class.name)
}

fn multiset(items: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
  let mut m = BTreeMap::new();
  for item in items.filter(|i| !i.is_empty()) {
    *m.entry(item).or_default() += 1;
  }
  m
}

/// The items of `a` not in `b` (as multisets).
fn minus(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> Vec<String> {
  a.iter()
    .flat_map(|(k, n)| std::iter::repeat_n(k.clone(), n.saturating_sub(*b.get(k).unwrap_or(&0))))
    .collect()
}

/// Every rule map of the shadow: each creator's, the block's, and what the rulings dropped.
fn rule_maps(shadow: &Tuples) -> impl Iterator<Item = &BTreeMap<String, String>> {
  shadow
    .creators
    .iter()
    .map(|c| &c.rules)
    .chain([&shadow.block_rules, &shadow.dropped])
}

/// The shadow rule that decided an item of `kind` with normalised text `text`, anywhere in the shadow (a creator's or the
/// block's): the item of that text (`moved …`), else — when the text is two or more of the shadow's items of that kind
/// joined, as today's one contact of two symbol lines `†Univ A‡Univ B` is the shadow's two — the rule of its first piece
/// (`overlaps …`). No looser overlap: a text that only shares a part with an item is the shadow's to explain.
fn rule_of(shadow: &Tuples, kind: &str, text: &str) -> Option<String> {
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
    let mut starts: Vec<usize> = if at > 0 && text[at..].starts_with("and") {
      vec![at, at + 3]
    } else {
      vec![at]
    };
    // (or the label the XML kept printed before it, the shadow's pieces split by those marks: `1INFN …2Dipartimento
    // …`, 1211.5032 — one or two digits, or a letter `a`-`h`)
    let rest = &text.as_bytes()[at..];
    let digits = rest
      .iter()
      .take(2)
      .take_while(|b| b.is_ascii_digit())
      .count();
    starts.extend((1..=digits).map(|k| at + k));
    if digits == 0 && rest.first().is_some_and(|b| (b'a'..=b'h').contains(b)) {
      starts.push(at + 1);
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
    _ => None,
  }
}

/// Whether `rest` is the marks `marks` written one after another (`12` the marks `1`, `2`; `ab`).
fn is_marks(rest: &str, marks: &[String]) -> bool {
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
fn xml_names(shadow: &Tuples, xml: &Tuples) -> Vec<String> {
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
        rule_of(shadow, "affiliation", &text),
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
        let rule = match (role, rule_of(shadow, role, text)) {
          ("url", None) => rule_of(shadow, "email", text).map(|r| format!("{r} role:url-as-email")),
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
    let without_orcids = |n: &String| {
      let mut n = norm(n);
      for id in &orcids {
        n = n.replace(id.as_str(), "");
      }
      n
    };
    let snotes = multiset(sc.notes.iter().map(without_orcids));
    let xnotes = multiset(xc.notes.iter().map(without_orcids));
    for text in minus(&snotes, &xnotes) {
      push(
        "note",
        sc.rules.get(&format!("note\t{text}")).cloned(),
        format!("{}: shadow-only {text}", sc.name),
      );
    }
    for text in minus(&xnotes, &snotes) {
      push(
        "note",
        rule_of(shadow, "note", &text),
        format!("{}: xml-only {text}", sc.name),
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
    push(
      "block",
      rule_of(shadow, role, text),
      format!("block: xml-only {item}"),
    );
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
fn align(a: &[String], b: &[String]) -> Vec<(usize, usize)> {
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

// ---- the driver ----

/// One paper's result.
#[derive(Debug, Clone, Default)]
pub struct PaperRow {
  pub key:           String,
  /// `agree`, `disagree`, `no-block`, `held`, `missing` (no dump or no XML), `unreadable`
  pub status:        String,
  pub family:        String,
  pub blocks:        usize,
  pub disagreements: Vec<Disagreement>,
  /// the shadow's tuples, for `<prefix>.shadow.jsonl` (disagreeing papers: the PDF checks read them)
  pub shadow:        Option<Tuples>,
}

impl PaperRow {
  fn unclassified(&self) -> bool { self.disagreements.iter().any(|d| !d.classified) }
}

/// Compare one paper directory (`out.xml` and `out.xml.fmshadow.json`).
pub fn compare_paper(root: &Path, key: &str, re_resolve: bool) -> PaperRow {
  let dir = root.join(key);
  let mut row = PaperRow {
    key: key.to_string(),
    ..PaperRow::default()
  };
  let (Ok(json), Ok(xml)) = (
    fs::read_to_string(dir.join("out.xml.fmshadow.json")),
    fs::read(dir.join("out.xml")),
  ) else {
    row.status = "missing".into();
    return row;
  };
  let Ok(dump) = serde_json::from_str::<Value>(&json) else {
    row.status = "unreadable".into();
    return row;
  };
  let blocks: Vec<&Value> = dump["entries"]
    .as_array()
    .into_iter()
    .flatten()
    .filter(|e| e["kind"] == "author" && e["status"] == "events")
    .collect();
  row.blocks = blocks.len();
  row.family = blocks
    .first()
    .map(|b| {
      // (`latexml.sty`: a dump of a binary that let the binding loading the kernel's pool own the kernel `\author`)
      let binding = s(&b["binding"]);
      if binding.is_empty() || binding == "latexml.sty" {
        "kernel".to_string()
      } else {
        binding
      }
    })
    .unwrap_or_else(|| "none".into());
  if blocks.is_empty() {
    row.status = "no-block".into();
    return row;
  }
  if blocks.iter().any(|b| b["errors"].as_u64().unwrap_or(0) > 0) {
    row.status = "held".into();
    return row;
  }
  let resolved = if re_resolve {
    fm_resolve(&json_inputs(&dump))
  } else {
    json_resolved(&dump["resolved"])
  };
  let xml = match xml_tuples(&xml) {
    Ok(t) => t,
    Err(_) => {
      row.status = "unreadable".into();
      return row;
    },
  };
  let shadow = shadow_tuples(&resolved);
  row.disagreements = compare(&shadow, &xml);
  row.status = if row.disagreements.is_empty() {
    "agree"
  } else {
    "disagree"
  }
  .into();
  if !row.disagreements.is_empty() {
    row.shadow = Some(shadow);
  }
  row
}

/// Run the comparison over `root` and write the reports.
pub fn run(
  root: &Path,
  keys: Option<&Path>,
  prefix: &Path,
  jobs: usize,
  re_resolve: bool,
) -> std::io::Result<i32> {
  let keys: Vec<String> = match keys {
    Some(path) => fs::read_to_string(path)?
      .lines()
      .map(|l| l.split('\t').next().unwrap_or("").trim().replace('/', ""))
      .filter(|l| !l.is_empty())
      .collect(),
    None => discover(root, 2).keys,
  };
  let rows = par_map(
    &keys,
    jobs,
    |key| compare_paper(root, key, re_resolve),
    |key| PaperRow {
      key: key.clone(),
      status: "panic".into(),
      ..PaperRow::default()
    },
    |_| {},
  );
  let path = |suffix: &str| {
    let mut p = prefix.as_os_str().to_owned();
    p.push(suffix);
    PathBuf::from(p)
  };
  let mut tsv = fs::File::create(path(".tsv"))?;
  writeln!(
    tsv,
    "key\tfamily\tstatus\tblocks\tunclassified\tclassified\tkinds\trules"
  )?;
  let mut dis = fs::File::create(path(".disagree.tsv"))?;
  let mut shadow_out = fs::File::create(path(".shadow.jsonl"))?;
  writeln!(dis, "key\tfamily\tkind\tclassified\trule\tdetail\tclass")?;
  #[derive(Default)]
  struct Fam {
    papers:       usize,
    blocks:       usize,
    agree:        usize,
    classified:   usize,
    unclassified: usize,
    no_block:     usize,
    held:         usize,
    other:        usize,
  }
  let mut fams: BTreeMap<String, Fam> = BTreeMap::new();
  let mut kinds: BTreeMap<(String, String, bool), (usize, usize)> = BTreeMap::new();
  for row in &rows {
    let un = row.disagreements.iter().filter(|d| !d.classified).count();
    let cl = row.disagreements.len() - un;
    let mut k: Vec<&str> = row.disagreements.iter().map(|d| d.kind).collect();
    k.sort_unstable();
    k.dedup();
    let mut r: Vec<&str> = row.disagreements.iter().map(|d| d.rule.as_str()).collect();
    r.sort_unstable();
    r.dedup();
    writeln!(
      tsv,
      "{}\t{}\t{}\t{}\t{un}\t{cl}\t{}\t{}",
      row.key,
      row.family,
      row.status,
      row.blocks,
      k.join(","),
      r.join(";")
    )?;
    let mut seen: Vec<(String, String, bool)> = Vec::new();
    for d in &row.disagreements {
      writeln!(
        dis,
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        row.key,
        row.family,
        d.kind,
        d.classified,
        d.rule,
        d.detail.replace(['\t', '\n'], " "),
        d.class.unwrap_or("")
      )?;
      let key = (
        d.kind.to_string(),
        d.rule
          .trim_start_matches("moved ")
          .trim_start_matches("overlaps ")
          .to_string(),
        d.classified,
      );
      let entry = kinds.entry(key.clone()).or_default();
      entry.1 += 1;
      if !seen.contains(&key) {
        entry.0 += 1;
        seen.push(key);
      }
    }
    if let Some(shadow) = &row.shadow {
      let creators: Vec<Value> = shadow
        .creators
        .iter()
        .map(|c| {
          serde_json::json!({
            "name": c.name,
            "affiliations": c.affiliations,
            "emails": c.contacts.iter().filter(|(r, _)| r == "email").map(|(_, t)| t.clone()).collect::<Vec<_>>(),
            "contacts": c.contacts,
            "notes": c.notes,
            "rules": c.rules,
          })
        })
        .collect();
      writeln!(
        shadow_out,
        "{}",
        serde_json::json!({ "key": row.key, "creators": creators, "block": shadow.block })
      )?;
    }
    let fam = fams.entry(row.family.clone()).or_default();
    fam.papers += 1;
    fam.blocks += row.blocks;
    match row.status.as_str() {
      "agree" => fam.agree += 1,
      "disagree" if row.unclassified() => fam.unclassified += 1,
      "disagree" => fam.classified += 1,
      "no-block" => fam.no_block += 1,
      "held" => fam.held += 1,
      _ => fam.other += 1,
    }
  }
  let mut summary = fs::File::create(path(".summary.txt"))?;
  writeln!(
    summary,
    "ab_audit fmshadow root={} papers={} re_resolve={re_resolve}",
    root.display(),
    rows.len()
  )?;
  writeln!(
    summary,
    "family\tpapers\tblocks\tagree\tdisagree_classified\tdisagree_unclassified\tno_block\theld\tother\tgate_unclassified_pct"
  )?;
  let mut ranked: Vec<(&String, &Fam)> = fams.iter().collect();
  ranked.sort_by(|a, b| b.1.papers.cmp(&a.1.papers).then(a.0.cmp(b.0)));
  let mut total = Fam::default();
  for (name, f) in &ranked {
    let counted = f.agree + f.classified + f.unclassified;
    let pct = if counted > 0 {
      100.0 * f.unclassified as f64 / counted as f64
    } else {
      0.0
    };
    writeln!(
      summary,
      "{name}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{pct:.1}",
      f.papers, f.blocks, f.agree, f.classified, f.unclassified, f.no_block, f.held, f.other
    )?;
    total.papers += f.papers;
    total.blocks += f.blocks;
    total.agree += f.agree;
    total.classified += f.classified;
    total.unclassified += f.unclassified;
    total.no_block += f.no_block;
    total.held += f.held;
    total.other += f.other;
  }
  let counted = total.agree + total.classified + total.unclassified;
  writeln!(
    summary,
    "TOTAL\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.1}",
    total.papers,
    total.blocks,
    total.agree,
    total.classified,
    total.unclassified,
    total.no_block,
    total.held,
    total.other,
    if counted > 0 {
      100.0 * total.unclassified as f64 / counted as f64
    } else {
      0.0
    }
  )?;
  writeln!(summary, "\nxml_wrong_class\tpapers\twitnesses")?;
  for class in XML_WRONG {
    let papers = rows
      .iter()
      .filter(|r| r.disagreements.iter().any(|d| d.class == Some(class.name)))
      .count();
    writeln!(
      summary,
      "{}\t{papers}\t{}",
      class.name,
      class.witnesses.join(" ")
    )?;
  }
  writeln!(summary, "\nkind\trule\tclassified\tpapers\toccurrences")?;
  let mut kinds: Vec<_> = kinds.into_iter().collect();
  kinds.sort_by_key(|a| std::cmp::Reverse(a.1));
  for ((kind, rule, classified), (papers, n)) in kinds {
    writeln!(summary, "{kind}\t{rule}\t{classified}\t{papers}\t{n}")?;
  }
  Ok(i32::from(total.unclassified > 0))
}

#[cfg(test)]
mod tests {
  use super::*;

  const HEAD: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><title>T</title>"#;

  fn creator(name: &str, affiliations: &[&str], rule: &str) -> TCreator {
    let mut c = TCreator {
      name: name.into(),
      affiliations: affiliations.iter().map(|a| a.to_string()).collect(),
      ..TCreator::default()
    };
    for a in affiliations {
      c.rules
        .insert(format!("affiliation\t{}", norm(a)), rule.into());
    }
    c
  }

  /// Convert `tex` under the shadow switch: its XML and its dump.
  fn convert(tex: &str) -> (String, Value) {
    let (log, xml, json) = latexml::util::test::convert_with_setup_then(
      tex,
      Some("ar5iv.sty"),
      || latexml_engine::base_utilities::set_fm_shadow_override(Some(true)),
      |_| latexml_engine::base_utilities::fm_shadow_json(),
    );
    assert_eq!(
      (
        latexml::util::test::error_count(&log),
        latexml::util::test::warning_count(&log)
      ),
      (0, 0),
      "{log}"
    );
    (xml, serde_json::from_str(&json).expect("json"))
  }

  /// The comparator on two conversions: a marked block the shadow resolves as the XML has it agrees; the design's
  /// `dag` probe disagrees, one classified row per moved symbol line (ruling 2: today's XML has both lines on Cat Cole,
  /// the shadow each on its carriers) — and resolving the dump's events again gives the dump's own resolution.
  #[test]
  fn comparator_rows_on_an_agreeing_and_a_disagreeing_conversion() {
    let agreeing = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{1}$ \\and Bob Baker$^{2}$\\\\ $^1$Univ A\\\\ $^2$Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
    let (xml, dump) = convert(agreeing);
    let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
    assert_eq!(shadow, shadow_tuples(&fm_resolve(&json_inputs(&dump))));
    assert_eq!(
      compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml")),
      vec![]
    );
    let disagreeing = "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^{\\dag}$ \\and Bob Baker$^{\\ddag}$ \\and Cat Cole$^{\\dag}$\\\\ \\dag Univ A\\\\ \\ddag Univ B}\n\\maketitle\nBody.\n\\end{document}\n";
    let (xml, dump) = convert(disagreeing);
    let shadow = shadow_tuples(&json_resolved(&dump["resolved"]));
    assert_eq!(shadow, shadow_tuples(&fm_resolve(&json_inputs(&dump))));
    let rows = compare(&shadow, &xml_tuples(xml.as_bytes()).expect("xml"));
    assert!(
      !rows.is_empty() && rows.iter().all(|d| d.kind == "affiliation" && d.classified),
      "{rows:?}"
    );
    assert!(
      rows
        .iter()
        .any(|d| d.rule == "ruling2:symbol-line"
          && d.detail.starts_with("Ann Able: shadow-only univa")),
      "{rows:?}"
    );
  }

  #[test]
  fn names_are_compared_without_their_marks() {
    let xml = xml_tuples(
      format!(
        r#"{HEAD}<creator role="author"><personname>Ann Able<sup>1</sup></personname><contact role="affiliation"><sup>1</sup>Univ A</contact></creator></document>"#
      )
      .as_bytes(),
    )
    .expect("xml");
    let shadow = Tuples {
      creators: vec![creator("Ann Able", &["Univ A"], "ruling1:request")],
      ..Tuples::default()
    };
    assert_eq!(compare(&shadow, &xml), vec![]);
  }

  #[test]
  fn a_symbol_line_on_its_carriers_is_a_classified_disagreement() {
    // (the dag probe: today's XML puts both symbol lines on the last name; ruling 2 on each carrier)
    let xml = xml_tuples(
      format!(
        r#"{HEAD}<creator><personname>Ann Able<sup>†</sup></personname></creator><creator><personname>Cat Cole<sup>†</sup></personname><contact role="affiliation">†Univ A</contact></creator></document>"#
      )
      .as_bytes(),
    )
    .expect("xml");
    let shadow = Tuples {
      creators: vec![
        creator("Ann Able", &["Univ A"], "ruling2:symbol-line"),
        creator("Cat Cole", &["Univ A"], "ruling2:symbol-line"),
      ],
      ..Tuples::default()
    };
    let d = compare(&shadow, &xml);
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(
      (d[0].kind, d[0].classified, d[0].rule.as_str()),
      ("affiliation", true, "ruling2:symbol-line")
    );
  }

  #[test]
  fn a_name_split_by_a_reproducing_rule_is_unclassified() {
    let xml = xml_tuples(
      format!(r#"{HEAD}<creator><personname>Ann Able Bob Baker</personname></creator></document>"#)
        .as_bytes(),
    )
    .expect("xml");
    let mut a = creator("Ann Able", &[], "");
    a.rules.insert("name".into(), "line:names".into());
    let mut b = creator("Bob Baker", &[], "");
    // (no separator: a split no XML-wrong class explains)
    b.rules.insert("name".into(), "line:names".into());
    let d = compare(
      &Tuples {
        creators: vec![a, b],
        ..Tuples::default()
      },
      &xml,
    );
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(
      (d[0].kind, d[0].classified, d[0].rule.as_str()),
      ("name-split", false, "line:names")
    );
  }

  /// The block's items are compared (OD #159's nameless creator) and `rule_of` finds them there; a joined XML item is
  /// explained only by the shadow's pieces joined, the word "and" between them allowed.
  #[test]
  fn block_items_and_joined_pieces() {
    let mut shadow = Tuples {
      creators: vec![creator(
        "Ann Able",
        &["Univ A", "Univ B"],
        "ruling2:symbol-line",
      )],
      ..Tuples::default()
    };
    shadow.block.push(("email".into(), "office@x.org".into()));
    shadow.block_rules.insert(
      "email\tofficexorg".into(),
      "owner:block ruling3:one-per-print".into(),
    );
    assert_eq!(
      rule_of(&shadow, "email", "officexorg").as_deref(),
      Some("moved owner:block ruling3:one-per-print")
    );
    assert_eq!(
      rule_of(&shadow, "affiliation", "univaandunivb").as_deref(),
      Some("overlaps ruling2:symbol-line")
    );
    assert_eq!(rule_of(&shadow, "affiliation", "univaunivc"), None);
    let xml = Tuples {
      creators: vec![creator("Ann Able", &["Univ A", "Univ B"], "")],
      block: vec![("email".into(), "office@x.org".into())],
      ..Tuples::default()
    };
    assert_eq!(compare(&shadow, &xml), vec![]);
    let xml = Tuples {
      creators: vec![creator("Ann Able", &["Univ A", "Univ B"], "")],
      ..Tuples::default()
    };
    let d = compare(&shadow, &xml);
    assert_eq!(
      (d.len(), d[0].kind, d[0].classified),
      (1, "block", false),
      "{d:?}"
    );
  }

  /// FM S2a.2 r3 normalisation: a name's marks the XML holds as text (`R. Vasile 1`, 0807.4407) or as a floating
  /// superscript formula (`Dadhich${}^{a,\;b}$`, 1307.6166; `\,^{3}`, 0804.4669) are no text of the name.
  #[test]
  fn marks_as_text_or_formulas_are_no_name_text() {
    let xml = xml_tuples(
      format!(
        r#"{HEAD}<creator><personname>R. Vasile 1</personname></creator><creator><personname>Naresh Dadhich<Math tex="{{}}^{{a,\;b}}" text="x"><XMath><XMTok>a</XMTok></XMath></Math></personname></creator><creator><personname>Roberta Zambrini<Math tex="\,^{{3}}" text="x"><XMath><XMTok>3</XMTok></XMath></Math></personname></creator></document>"#
      )
      .as_bytes(),
    )
    .expect("xml");
    let mut a = creator("R. Vasile", &[], "");
    a.marks = vec!["1".into()];
    let shadow = Tuples {
      creators: vec![
        a,
        creator("Naresh Dadhich", &[], ""),
        creator("Roberta Zambrini", &[], ""),
      ],
      ..Tuples::default()
    };
    assert_eq!(compare(&shadow, &xml), vec![]);
  }

  /// FM S2a.2 r3 classes: names parted only by a mark or a note merged into one personname (1609.02129); a url whose text is an
  /// e-mail address read as an e-mail (r2 item 8, 1701.01829); a title's note on the first author (1603.01892); the
  /// addresses of a names line in the personname (2405.06319) — each disagreement classified, by its class or its tag.
  #[test]
  fn r3_classes_classify_their_disagreements() {
    let named = |name: &str, rule: &str| {
      let mut c = creator(name, &[], "");
      c.rules.insert("name".into(), rule.into());
      c
    };
    fn marked(mut c: TCreator, mark: &str) -> TCreator {
      c.marks = vec![mark.into()];
      c
    }
    let one = |body: &str| xml_tuples(format!("{HEAD}{body}</document>").as_bytes()).expect("xml");
    let shadow = Tuples {
      creators: vec![
        marked(named("C. Nisoli", "line:names"), "2"),
        named("C. Reichhardt", "line:names"),
      ],
      ..Tuples::default()
    };
    let d = compare(
      &shadow,
      &one("<creator><personname>C. Nisoli C. Reichhardt</personname></creator>"),
    );
    assert_eq!(
      (d.len(), d[0].class),
      (
        1,
        Some("names parted only by a mark or a note merged into one personname")
      ),
      "{d:?}"
    );
    let mut ann = named("Ann Able", "line:names");
    ann.contacts.push(("email".into(), "ann@x.org".into()));
    ann
      .rules
      .insert("email\tannxorg".into(), "api:annotate:last".into());
    let shadow = Tuples {
      creators: vec![ann.clone()],
      ..Tuples::default()
    };
    let d = compare(
      &shadow,
      &one(
        r#"<creator><personname>Ann Able</personname><contact role="url">ann@x.org</contact></creator>"#,
      ),
    );
    assert!(d.len() == 2 && d.iter().all(|d| d.classified), "{d:?}");
    let mut shadow = Tuples {
      creators: vec![named("Ann Able", "line:names")],
      ..Tuples::default()
    };
    shadow.block.push(("note".into(), "Supported.".into()));
    shadow.block_rules.insert(
      "note\tsupported".into(),
      "api:owner:block api:title-note".into(),
    );
    let d = compare(
      &shadow,
      &one(r#"<creator><personname>Ann Able</personname><note>Supported.</note></creator>"#),
    );
    assert!(
      d.len() == 2
        && d.iter().all(|d| d.class
          == Some("a note made in the title (before any author) given to the first author")),
      "{d:?}"
    );
    let shadow = Tuples {
      creators: vec![ann],
      ..Tuples::default()
    };
    let d = compare(
      &shadow,
      &one("<creator><personname>Ann Able (ann@x.org)</personname></creator>"),
    );
    assert!(
      !d.is_empty()
        && d.iter().all(|d| {
          d.class == Some("e-mail addresses printed in the names line read as creators or into the name before them")
        }),
      "{d:?}"
    );
  }

  /// A name disagreement is classified by an XML-wrong class only when the class explains every name: wide-space names
  /// merged into one personname (2312.06207) — but not when a name is also missing.
  #[test]
  fn xml_wrong_name_classes_explain_every_name() {
    let mut a = creator("Ann Able", &[], "");
    a.rules.insert("name".into(), "line:names".into());
    let mut b = creator("Bob Baker", &[], "");
    b.rules.insert("name".into(), "line:names sep:gap".into());
    let shadow = Tuples {
      creators: vec![a, b],
      ..Tuples::default()
    };
    let merged = xml_tuples(
      format!(
        r#"{HEAD}<creator><personname>Ann Able  Bob Baker</personname></creator></document>"#
      )
      .as_bytes(),
    )
    .expect("xml");
    let d = compare(&shadow, &merged);
    assert_eq!(
      (d.len(), d[0].classified, d[0].class),
      (1, true, Some(XML_WRONG[0].name)),
      "{d:?}"
    );
    let other = xml_tuples(
      format!(r#"{HEAD}<creator><personname>Ann Able</personname></creator></document>"#)
        .as_bytes(),
    )
    .expect("xml");
    let d = compare(&shadow, &other);
    assert_eq!((d.len(), d[0].classified), (1, false), "{d:?}");
  }
}
