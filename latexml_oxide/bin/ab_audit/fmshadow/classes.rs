//! The PDF-confirmed classes of disagreement where today's XML is wrong and the shadow right ([`XML_WRONG`]), each
//! listed by name with its witnesses.

use super::*;

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
pub(super) enum NameFault {
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

/// Whether an XML-only affiliation (`<name>: xml-only <text>`) is one of the shadow's affiliations of that creator with
/// legend lines the shadow reads as ruling 2's symbol lines welded on after it (`… 55441, USA.∗These authors
/// contributed equally to this work.†Corresponding author: yi@virginia.edu`, 2305.13575): the rest of its text is a
/// sequence of those lines' notes (kept, or dropped by ruling 4 once their e-mail became a contact).
pub(super) fn legends_welded_on(shadow: &Tuples, detail: &str) -> bool {
  let Some((name, text)) = detail.split_once(": xml-only ") else {
    return false;
  };
  let legends: Vec<String> = shadow
    .creators
    .iter()
    .flat_map(|c| {
      c.notes
        .iter()
        .map(|n| norm(n))
        .filter(|n| {
          c.rules
            .get(&format!("note\t{n}"))
            .is_some_and(|r| r.contains("ruling2:symbol-line"))
        })
        .collect::<Vec<_>>()
    })
    .chain(
      shadow
        .dropped
        .iter()
        .filter(|(_, rule)| rule.contains("ruling2:symbol-line"))
        .filter_map(|(key, _)| key.strip_prefix("note\t").map(str::to_string)),
    )
    .chain(
      shadow
        .block
        .iter()
        .filter(|(role, x)| {
          role == "note"
            && shadow
              .block_rules
              .get(&format!("note\t{}", norm(x)))
              .is_some_and(|r| r.contains("ruling2:symbol-line"))
        })
        .map(|(_, x)| norm(x)),
    )
    .filter(|l| !l.is_empty())
    .collect();
  // (an e-mail legend's address, its label words before it in the XML: "Corresponding author:", "To whom
  // correspondence should be addressed; E-mail:", 2012.15253)
  let addresses: Vec<String> = shadow
    .creators
    .iter()
    .flat_map(|c| {
      c.contacts
        .iter()
        .filter(|(role, _)| role == "email")
        .map(|(_, t)| norm(t))
        .filter(|t| {
          c.rules
            .get(&format!("email\t{t}"))
            .is_some_and(|r| r.contains("ruling2:symbol-line"))
        })
        .collect::<Vec<_>>()
    })
    .filter(|t| !t.is_empty())
    .collect();
  if legends.is_empty() && addresses.is_empty() {
    return false;
  }
  // (the rest as a sequence of legends, any order)
  let joined = |rest: &str| {
    let n = rest.len();
    let mut reach = vec![false; n + 1];
    reach[0] = true;
    for at in 0..n {
      if !reach[at] {
        continue;
      }
      for legend in &legends {
        if rest[at..].starts_with(legend.as_str()) {
          reach[at + legend.len()] = true;
        }
      }
      for address in &addresses {
        if let Some(j) = rest[at..].find(address.as_str()) {
          let label = &rest[at..at + j];
          if label.len() <= 60 && label.chars().all(char::is_alphabetic) {
            reach[at + j + address.len()] = true;
          }
        }
      }
    }
    reach[n]
  };
  shadow.creators.iter().filter(|c| c.name == name).any(|c| {
    c.affiliations.iter().any(|a| {
      let a = norm(a);
      !a.is_empty() && text.len() > a.len() && text.starts_with(&a) && joined(&text[a.len()..])
    })
  })
}

/// Whether a text is e-mail addresses alone (brackets, braces and separators aside).
pub(super) fn is_addresses(text: &str) -> bool {
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
pub(super) fn names_explained(shadow: &Tuples, xml: &Tuples, fault: NameFault) -> bool {
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
pub(super) fn block_in_one_name(shadow: &Tuples, xml: &Tuples) -> bool {
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
pub(super) fn names_lost_to_affiliations(shadow: &Tuples, xml: &Tuples) -> Vec<String> {
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
    name:      "legend lines (a symbol and its note) welded onto the affiliation above them",
    witnesses: &[
      "2305.13575",
      "2012.15253",
      "2110.08090",
      "2502.01939",
      "2601.00327",
      "2607.22634",
      "2607.22703",
    ],
    applies:   |c| c.kind == "affiliation" && legends_welded_on(c.shadow, c.detail),
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
pub(super) fn xml_wrong_class(context: &Context<'_>) -> Option<&'static str> {
  XML_WRONG
    .iter()
    .find(|class| (class.applies)(context))
    .map(|class| class.name)
}
