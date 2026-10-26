//! `ab_audit fmshadow ROOT`: the frontmatter observer's shadow resolution (`<dest>.fmshadow.json`, written under
//! `LATEXML_FM_SHADOW=1`; OXIDIZED_DESIGN_DIVERGENCES #837) against the frontmatter of the XML beside it, per paper.
//!
//! Both sides are read into the same tuples: per creator its name (marks stripped: an `ltx:sup` in the personname is
//! a mark), its affiliations (`affiliation`/`address` contacts), its other contacts by role, its notes; and the
//! author block's own (the XML's trailing nameless creator, OD #159), compared as a multiset too. Each disagreement is
//! classified by its kind (`name-split`, `affiliation`, `contact-owner`, `contact`, `note`, `block`), the family (the
//! policy key's binding) and the shadow rule that decided it; a disagreement is **classified** when that rule holds an
//! adopted ruling's change tag ([`CHANGE_TAGS`](latexml_engine::base_utilities::FM_CHANGE_TAGS): the output S2b is
//! meant to change) or it falls in a PDF-confirmed XML-wrong class ([`XML_WRONG`], listed by name with its witnesses), and **unclassified** otherwise: a place the
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
  FmAssoc, FmEvent, FmEventKind, FmInput, FmResolved, FmSepKind, fm_addresses_in, fm_creator_call,
  fm_resolve,
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
  pub creators:        Vec<TCreator>,
  /// the author block's own contacts and notes, `(role, text)` — role `affiliation` for an affiliation or address,
  /// `note` for a note ([`block_role`])
  pub block:           Vec<(String, String)>,
  /// the shadow's rules for the block's items: `role\ttext` → rule (the XML's empty)
  pub block_rules:     BTreeMap<String, String>,
  /// every rule of the block's items of one key, in their order ([`TCreator::all_rules`])
  pub block_all_rules: BTreeMap<String, Vec<String>>,
  /// the shadow's rules for what the rulings dropped (an emptied note, ruling 4): `note\ttext` → rule
  pub dropped:         BTreeMap<String, String>,
  /// the shadow's entries that unmarked lines continued, each its lines normalised (`FmResolved::continued`; `None` for
  /// a dump made before FM S2a.2 r8)
  pub continued:       Option<Vec<Vec<String>>>,
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
  /// every rule of the shadow's items of one key (`role\ttext`), in their order: the copies of one text (ruling 3: an
  /// address in a note and on the line under it, 2509.23109; 1910.01401's two ORCID copies) each with its own, which
  /// the comparison gives the copies the XML lacks one by one (`surplus_rules` — review revfms2a2_8 finding 3b)
  pub all_rules:    BTreeMap<String, Vec<String>>,
  /// the shadow's marks shown on the name (its requests, prefixes aside), normalised
  pub marks:        Vec<String>,
  /// the XML's notes set inside the personname (a footnote on the name: the text of a `\footnotemark` it shows,
  /// `\footnotetext[1]{…}` after `\maketitle`, 1108.5155), compared as the creator's notes are
  pub name_notes:   Vec<String>,
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

/// The text of `node` without its `ltx:sup` descendants (a name's or an affiliation's marks), its notes, nor a note's
/// `ltx:tags` (its number and reference names, "* footnote * footnote Footnote …"); a formula that is a mark as
/// [`math_mark_text`] reads it.
fn text_without_marks(node: Node<'_, '_>) -> String { text_without(node, false) }

/// An affiliation's text as [`text_without_marks`], a formula's superscript a mark too (`OR $97403^{1}$`,
/// `Ul. Ainy $299^A$`: the base its text — the lexer's reading of a formula, `events.rs` `walk_math`; 2203.01892,
/// 0707.4047).
fn affiliation_text(node: Node<'_, '_>) -> String { text_without(node, true) }

/// A formula `<base>^{…}` (or `<base>^x`) whose base is a word or a number: that base.
fn math_base(math: Node<'_, '_>) -> Option<String> {
  let tex = math.attribute("tex")?.trim();
  let (base, _) = tex.split_once('^')?;
  let base = base.trim();
  (!base.is_empty()
    && base
      .chars()
      .all(|c| c.is_alphanumeric() || c == '.' || c == '-'))
  .then(|| base.to_string())
}

/// A superscript of lowercase letters glued on both sides to letters (`James M<sup>c</sup>Kernan`, math0610203): part
/// of the word, as the frontmatter observer's lexer reads it.
fn inside_a_word(sup: Node<'_, '_>) -> bool {
  let text: String = sup
    .descendants()
    .filter(|n| n.is_text())
    .filter_map(|n| n.text())
    .collect();
  let glued = |side: Option<Node<'_, '_>>, last: bool| {
    side
      .filter(|n| n.is_text())
      .and_then(|n| n.text())
      .is_some_and(|t| {
        let c = if last {
          t.chars().next_back()
        } else {
          t.chars().next()
        };
        c.is_some_and(char::is_alphabetic)
      })
  };
  (1..=2).contains(&text.chars().count())
    && text.chars().all(char::is_lowercase)
    && glued(sup.prev_sibling(), true)
    && glued(sup.next_sibling(), false)
}

fn text_without(node: Node<'_, '_>, scripts: bool) -> String {
  fn walk(node: Node<'_, '_>, scripts: bool, out: &mut String) {
    for child in node.children() {
      if child.is_text() {
        out.push_str(child.text().unwrap_or(""));
      } else if is_ltx(child, "Math")
        && let Some(text) =
          math_mark_text(child).or_else(|| scripts.then(|| math_base(child)).flatten())
      {
        out.push_str(&text);
      } else if is_ltx(child, "sup") && inside_a_word(child) {
        // (a lowercase superscript inside a word is the word's letters, no mark: `M\textsuperscript{c}Kernan`)
        walk(child, scripts, out);
      } else if child.is_element()
        && !is_ltx(child, "sup")
        && !is_ltx(child, "note")
        && !is_ltx(child, "tags")
      {
        walk(child, scripts, out);
      }
      if is_ltx(child, "break") {
        out.push(' ');
      }
    }
  }
  let mut raw = String::new();
  walk(node, scripts, &mut raw);
  collapse(&raw)
}

/// The XML's tuples.
pub fn xml_tuples(xml: &[u8]) -> Result<Tuples, String> {
  let doc = parse(xml)?;
  let mut out = Tuples::default();
  // (a note set in the title is the document's, as the shadow's title note in the block is: aastex's
  // `\title{…\altaffilmark{1}}` and its `\altaffiltext{1}{…}`, 0801.1670)
  for note in doc
    .root_element()
    .descendants()
    .filter(|n| is_ltx(*n, "note") && n.ancestors().any(|a| is_ltx(a, "title")))
  {
    out
      .block
      .push(("note".to_string(), text_without_marks(note)));
  }
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
        // (a footnote set in the name, its text without its tags, explains the shadow's note of the creator: revtex's
        // `\author{Hui-Yun Cao$^{1}$\footnotemark[1]}` and `\footnotetext[1]{E-mail: …}`, 2503.10108)
        for note in child.descendants().filter(|n| is_ltx(*n, "note")) {
          if !note.ancestors().skip(1).any(|a| is_ltx(a, "note")) {
            creator.name_notes.push(text_without_marks(note));
          }
        }
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
        // (an ORCID link inside an e-mail contact, `\email{… \orcidlink{…}}`, is the creator's ORCID too, as the shadow
        // reads it: 2402.04675)
        if role == "email" {
          for link in child.descendants().filter(|n| is_ltx(*n, "ref")) {
            if let Some(href) = link.attribute("href").filter(|h| h.contains("orcid.org/")) {
              creator
                .contacts
                .push(("orcid".into(), orcid_id(href).to_string()));
            }
          }
        }
        let mut text = if role == "affiliation" || role == "address" {
          affiliation_text(child)
        } else {
          text_without_marks(child)
        };
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

/// An item's rule, by its kind and text; of two items of one kind and text (ruling 3: one per print event — an address in
/// a note and on the line under it, 2509.23109), the rule of a ruling's change, the reason the shadow holds the second,
/// whatever their order (FM S2a.2 r8 sorts a creator's contacts by source).
/// Every rule is kept as well, by key (`TCreator::all_rules`).
fn assoc_rule(creator: &mut TCreator, kind: &str, assoc: &FmAssoc) {
  let text = if kind == "orcid" {
    orcid_id(&assoc.text)
  } else {
    &assoc.text
  };
  let key = format!("{kind}\t{}", norm(text));
  creator
    .all_rules
    .entry(key.clone())
    .or_default()
    .push(assoc.rule.clone());
  let rules = &mut creator.rules;
  if rules
    .get(&key)
    .is_some_and(|held| is_change_rule(held) && !is_change_rule(&assoc.rule))
  {
    return;
  }
  rules.insert(key, assoc.rule.clone());
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
      assoc_rule(&mut creator, "affiliation", a);
    }
    for a in &c.contacts {
      if is_note_role(&a.role) {
        creator.notes.push(a.text.clone());
        assoc_rule(&mut creator, "note", a);
      } else {
        let text = if a.role == "orcid" {
          orcid_id(&a.text).to_string()
        } else {
          a.text.clone()
        };
        creator.contacts.push((a.role.clone(), text));
        assoc_rule(&mut creator, &a.role, a);
      }
    }
    for a in &c.notes {
      creator.notes.push(a.text.clone());
      assoc_rule(&mut creator, "note", a);
    }
    out.creators.push(creator);
  }
  out.continued = resolved.continued.as_ref().map(|all| {
    all
      .iter()
      .map(|lines| lines.iter().map(|l| norm(l)).collect())
      .collect()
  });
  for a in &resolved.dropped {
    out
      .dropped
      .insert(format!("note\t{}", norm(&a.text)), a.rule.clone());
  }
  for a in &resolved.block {
    let role = block_role(&a.role);
    let key = format!("{role}\t{}", norm(&a.text));
    out
      .block_all_rules
      .entry(key.clone())
      .or_default()
      .push(a.rule.clone());
    out.block_rules.insert(key, a.rule.clone());
    out.block.push((role, a.text.clone()));
  }
  out
}

mod classes;
mod compare;
mod driver;
mod dump;
#[cfg(test)]
mod tests;

pub(crate) use classes::*;
pub(crate) use compare::*;
pub(crate) use driver::*;
pub(crate) use dump::*;
