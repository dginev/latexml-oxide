//! Frontmatter tuples (design §5 Stage 0): per creator its role, name, contacts and notes, plus the
//! document's title, abstract and keywords; and the comparison of two documents' tuples.
//!
//! A creator lost or gained, a contact emptied, a name now empty or an address newly split into runs is
//! what sample statistics missed (63w4: an orcid slot swallowed the next author of 2605.02715; 64k5: 41
//! empty contacts; 64zc r1: a typewriter address split by a `$\sim$`, 1601.02576, linked to one character).

use roxmltree::{Document, Node};

use crate::facts::{collapse, is_ltx, text_of};

/// One `ltx:contact` of a creator.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Contact {
  pub role:  String,
  /// All its text, collapsed.
  pub text:  String,
  /// The `href`s inside it (an explicit `\href`/`\url`).
  pub hrefs: Vec<String>,
  /// The link the HTML gives an email/url/orcid contact without an `ltx:ref`: the XSLT `contact-address`
  /// rule (latexml_post/resources/XSLT/LaTeXML-structure-xhtml.xsl:870-884) with its role's prefix.
  pub link:  Option<String>,
  /// A typewriter run beside bare text or another run: the shape whose link the XSLT took from the first
  /// text alone before 64zc (1601.02576, 1601.02586, 1601.03451, 1601.06529).
  pub split: bool,
}

/// One `ltx:creator`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Creator {
  pub role:     String,
  pub name:     String,
  pub contacts: Vec<Contact>,
  /// `(role, text)` of each `ltx:note` child.
  pub notes:    Vec<(String, String)>,
}

/// The frontmatter of one document.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Front {
  pub creators:      Vec<Creator>,
  pub title:         String,
  pub abstract_text: String,
  pub keywords:      String,
}

impl Front {
  /// Contacts with neither text nor an `href`.
  pub fn empty_contacts(&self) -> usize {
    self
      .creators
      .iter()
      .flat_map(|c| &c.contacts)
      .filter(|c| c.text.is_empty() && c.hrefs.is_empty())
      .count()
  }
  /// Creators whose name is empty.
  pub fn nameless(&self) -> usize { self.creators.iter().filter(|c| c.name.is_empty()).count() }
  /// Contacts split into runs.
  pub fn split_contacts(&self) -> usize {
    self
      .creators
      .iter()
      .flat_map(|c| &c.contacts)
      .filter(|c| c.split)
      .count()
  }
  /// Contacts whose derived link is a fragment of their text: at most 3 characters of an address at least
  /// three times as long (after its role's prefix).
  pub fn fragment_links(&self) -> usize {
    self
      .creators
      .iter()
      .flat_map(|c| &c.contacts)
      .filter(|c| {
        c.link.as_ref().is_some_and(|link| {
          let address = link
            .strip_prefix("mailto:")
            .or_else(|| link.strip_prefix("https://orcid.org/"))
            .unwrap_or(link);
          let (n, m) = (address.chars().count(), c.text.chars().count());
          n <= 3 && m >= 3 * n.max(1)
        })
      })
      .count()
  }
}

fn attr(node: Node<'_, '_>, name: &str) -> String { node.attribute(name).unwrap_or("").to_string() }

fn is_typewriter_run(node: Node<'_, '_>) -> bool {
  is_ltx(node, "text")
    && node
      .attribute("font")
      .is_some_and(|f| f.contains("typewriter"))
}

/// The XSLT `contact-address` value (LaTeXML-structure-xhtml.xsl:870-884): with element children that are
/// all typewriter runs, all the text; else the first non-blank text child; else the first non-blank text
/// anywhere inside.
fn contact_address(contact: Node<'_, '_>) -> String {
  let elements: Vec<_> = contact.children().filter(Node::is_element).collect();
  if !elements.is_empty() && elements.iter().all(|e| is_typewriter_run(*e)) {
    return text_of(contact);
  }
  let blank = |n: &Node<'_, '_>| n.text().is_none_or(|t| t.trim_ascii().is_empty());
  if let Some(t) = contact.children().find(|n| n.is_text() && !blank(n)) {
    return collapse(t.text().unwrap_or(""));
  }
  contact
    .descendants()
    .find(|n| n.is_text() && !blank(n))
    .map(|t| collapse(t.text().unwrap_or("")))
    .unwrap_or_default()
}

fn read_contact(node: Node<'_, '_>) -> Contact {
  let role = attr(node, "role");
  let hrefs: Vec<String> = node
    .descendants()
    .filter_map(|d| d.attribute("href"))
    .map(collapse)
    .collect();
  // "Already formatted as link?": email and url test `.//ltx:ref`, orcid a child `ltx:ref` only
  // (LaTeXML-structure-xhtml.xsl, the contact[@role='email'|'orcid'|'url'] inner templates).
  let has_ref = if role == "orcid" {
    node.children().any(|d| is_ltx(d, "ref"))
  } else {
    node.descendants().any(|d| is_ltx(d, "ref"))
  };
  let prefix = match role.as_str() {
    "email" => Some("mailto:"),
    "url" => Some(""),
    "orcid" => Some("https://orcid.org/"),
    _ => None,
  };
  let link = prefix
    .filter(|_| !has_ref)
    .map(|p| format!("{p}{}", contact_address(node)));
  let elements: Vec<_> = node.children().filter(Node::is_element).collect();
  let bare_text = node
    .children()
    .any(|n| n.is_text() && n.text().is_some_and(|t| !t.trim_ascii().is_empty()));
  let split = elements.iter().any(|e| is_typewriter_run(*e)) && (bare_text || elements.len() > 1);
  Contact {
    role,
    text: text_of(node),
    hrefs,
    link,
    split,
  }
}

/// Read the frontmatter tuples of a document.
pub fn read_front(doc: &Document<'_>) -> Front {
  let root = doc.root_element();
  let mut front = Front {
    title: root
      .children()
      .find(|n| is_ltx(*n, "title"))
      .map(text_of)
      .unwrap_or_default(),
    ..Front::default()
  };
  let mut abstracts = Vec::new();
  let mut keywords = Vec::new();
  for node in root.descendants() {
    if is_ltx(node, "creator") {
      let mut creator = Creator {
        role: attr(node, "role"),
        ..Creator::default()
      };
      let mut names = Vec::new();
      for child in node.children() {
        if is_ltx(child, "personname") {
          names.push(text_of(child));
        } else if is_ltx(child, "contact") {
          creator.contacts.push(read_contact(child));
        } else if is_ltx(child, "note") {
          creator.notes.push((attr(child, "role"), text_of(child)));
        }
      }
      creator.name = names.join(" | ");
      front.creators.push(creator);
    } else if is_ltx(node, "abstract") {
      abstracts.push(text_of(node));
    } else if is_ltx(node, "keywords") || is_ltx(node, "classification") {
      keywords.push(text_of(node));
    }
  }
  front.abstract_text = abstracts.join(" | ");
  front.keywords = keywords.join(" | ");
  front
}

/// What changed between two documents' frontmatter.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FrontDiff {
  /// Flag classes (`creator-lost`, `contact-emptied` …).
  pub flags: Vec<String>,
  /// One line per change, for the detail file and the TSV.
  pub notes: Vec<String>,
}

/// Align two name lists by their longest common subsequence: the matched `(a, b)` index pairs in order.
fn align(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
  let (n, m) = (a.len(), b.len());
  if n.saturating_mul(m) > 16_000_000 {
    // Thousands of authors on both sides: positional.
    return (0..n.min(m))
      .filter(|&i| a[i] == b[i])
      .map(|i| (i, i))
      .collect();
  }
  let mut table = vec![0u32; (n + 1) * (m + 1)];
  let at = |i: usize, j: usize| i * (m + 1) + j;
  for i in (0..n).rev() {
    for j in (0..m).rev() {
      table[at(i, j)] = if a[i] == b[j] {
        table[at(i + 1, j + 1)] + 1
      } else {
        table[at(i + 1, j)].max(table[at(i, j + 1)])
      };
    }
  }
  let (mut i, mut j, mut pairs) = (0, 0, Vec::new());
  while i < n && j < m {
    if a[i] == b[j] {
      pairs.push((i, j));
      i += 1;
      j += 1;
    } else if table[at(i + 1, j)] >= table[at(i, j + 1)] {
      i += 1;
    } else {
      j += 1;
    }
  }
  pairs
}

fn show_contact(c: &Contact) -> String {
  let mut s = format!("{}={:?}", c.role, c.text);
  if !c.hrefs.is_empty() {
    s.push_str(&format!(" href={:?}", c.hrefs));
  }
  if let Some(link) = &c.link {
    s.push_str(&format!(" link={link:?}"));
  }
  if c.split {
    s.push_str(" split");
  }
  s
}

/// Compare the creators `a` and `b` paired by name or position; push flags and notes.
fn diff_creator(a: &Creator, b: &Creator, out: &mut FrontDiff) {
  if a == b {
    return;
  }
  let who = if a.name == b.name {
    format!("{:?}", a.name)
  } else {
    format!("{:?} -> {:?}", a.name, b.name)
  };
  if a.name != b.name && b.name.is_empty() {
    out.flags.push("creator-nameless".into());
  }
  // Contacts paired by role and their index within the role.
  let mut roles: Vec<&str> = a
    .contacts
    .iter()
    .chain(&b.contacts)
    .map(|c| c.role.as_str())
    .collect();
  roles.sort_unstable();
  roles.dedup();
  for role in roles {
    let ca: Vec<&Contact> = a.contacts.iter().filter(|c| c.role == role).collect();
    let cb: Vec<&Contact> = b.contacts.iter().filter(|c| c.role == role).collect();
    for k in 0..ca.len().max(cb.len()) {
      match (ca.get(k), cb.get(k)) {
        (Some(x), Some(y)) if x == y => {},
        (Some(x), Some(y)) => {
          let emptied =
            !(x.text.is_empty() && x.hrefs.is_empty()) && y.text.is_empty() && y.hrefs.is_empty();
          if emptied {
            out.flags.push("contact-emptied".into());
          }
          if y.split && !x.split {
            out.flags.push("contact-split".into());
          }
          if x.link != y.link || x.hrefs != y.hrefs {
            out.flags.push("contact-link-changed".into());
          }
          out.notes.push(format!(
            "creator {who}: contact {} -> {}",
            show_contact(x),
            show_contact(y)
          ));
        },
        (Some(x), None) => {
          out.flags.push("contact-lost".into());
          out
            .notes
            .push(format!("creator {who}: contact lost {}", show_contact(x)));
        },
        (None, Some(y)) => {
          if y.text.is_empty() && y.hrefs.is_empty() {
            out.flags.push("contact-emptied".into());
          }
          out
            .notes
            .push(format!("creator {who}: contact added {}", show_contact(y)));
        },
        (None, None) => {},
      }
    }
  }
  if a.role != b.role {
    out
      .notes
      .push(format!("creator {who}: role {:?} -> {:?}", a.role, b.role));
  }
  if a.notes != b.notes {
    out.notes.push(format!(
      "creator {who}: notes {:?} -> {:?}",
      a.notes, b.notes
    ));
  }
  out.flags.push("creator-changed".into());
  if a.name != b.name {
    out.notes.push(format!("creator renamed {who}"));
  }
}

/// Compare two documents' frontmatter tuples.
pub fn diff_front(a: &Front, b: &Front) -> FrontDiff {
  let mut out = FrontDiff::default();
  if a == b {
    return out;
  }
  let na: Vec<&str> = a.creators.iter().map(|c| c.name.as_str()).collect();
  let nb: Vec<&str> = b.creators.iter().map(|c| c.name.as_str()).collect();
  let pairs = align(&na, &nb);
  // Walk the gaps between matched creators: pair what is left in each gap by position, the rest is
  // lost (A) or added (B).
  let mut anchors = pairs.clone();
  anchors.push((na.len(), nb.len()));
  let (mut i, mut j) = (0, 0);
  for (ai, bj) in anchors {
    let (gap_a, gap_b) = (ai - i, bj - j);
    for k in 0..gap_a.min(gap_b) {
      diff_creator(&a.creators[i + k], &b.creators[j + k], &mut out);
    }
    for k in gap_b..gap_a {
      out.flags.push("creator-lost".into());
      out
        .notes
        .push(format!("creator lost {:?}", a.creators[i + k].name));
    }
    for k in gap_a..gap_b {
      let added = &b.creators[j + k];
      out.flags.push(
        if added.name.is_empty() {
          "creator-nameless"
        } else {
          "creator-added"
        }
        .into(),
      );
      out.notes.push(format!("creator added {:?}", added.name));
      if added
        .contacts
        .iter()
        .any(|c| c.text.is_empty() && c.hrefs.is_empty())
      {
        out.flags.push("contact-emptied".into());
      }
    }
    if ai < na.len() {
      diff_creator(&a.creators[ai], &b.creators[bj], &mut out);
    }
    (i, j) = (ai + 1, bj + 1);
  }
  if a.creators.len() != b.creators.len() {
    out.notes.insert(
      0,
      format!("creators {} -> {}", a.creators.len(), b.creators.len()),
    );
  }
  // Document-wide counts, so a change the pairing explains away still shows.
  if b.empty_contacts() > a.empty_contacts() {
    out.flags.push("contact-emptied".into());
    out.notes.push(format!(
      "empty contacts {} -> {}",
      a.empty_contacts(),
      b.empty_contacts()
    ));
  }
  if b.nameless() > a.nameless() {
    out.flags.push("creator-nameless".into());
  }
  if b.split_contacts() > a.split_contacts() {
    out.flags.push("contact-split".into());
    out.notes.push(format!(
      "split contacts {} -> {}",
      a.split_contacts(),
      b.split_contacts()
    ));
  }
  if b.fragment_links() > a.fragment_links() {
    out.flags.push("contact-link-fragment".into());
  }
  for (what, x, y) in [
    ("title", &a.title, &b.title),
    ("abstract", &a.abstract_text, &b.abstract_text),
    ("keywords", &a.keywords, &b.keywords),
  ] {
    if x != y {
      out.flags.push(if y.is_empty() {
        format!("{what}-lost")
      } else {
        format!("{what}-changed")
      });
      out.notes.push(format!("{what} {x:?} -> {y:?}"));
    }
  }
  out.flags.sort();
  out.flags.dedup();
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::facts::parse;

  fn front(xml: &str) -> Front { read_front(&parse(xml.as_bytes()).expect("well-formed")) }

  const HEAD: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><title>T</title>"#;

  #[test]
  fn a_creator_whose_contact_empties_is_flagged() {
    let a = front(&format!(
      r#"{HEAD}<creator role="author"><personname>Ann Able</personname>
      <contact role="email">ann@a.edu</contact></creator></document>"#
    ));
    let b = front(&format!(
      r#"{HEAD}<creator role="author"><personname>Ann Able</personname>
      <contact role="email"></contact></creator></document>"#
    ));
    let d = diff_front(&a, &b);
    assert!(d.flags.contains(&"contact-emptied".to_string()), "{d:?}");
    assert!(d.flags.contains(&"creator-changed".to_string()), "{d:?}");
  }

  #[test]
  fn a_swallowed_author_is_lost_not_renamed() {
    let a = front(&format!(
      r#"{HEAD}<creator role="author"><personname>Ann</personname><contact role="orcid">0000-1</contact></creator>
      <creator role="author"><personname>Bob</personname></creator>
      <creator role="author"><personname>Cy</personname></creator></document>"#
    ));
    let b = front(&format!(
      r#"{HEAD}<creator role="author"><personname>Ann</personname><contact role="orcid">Bob</contact></creator>
      <creator role="author"><personname>Cy</personname></creator></document>"#
    ));
    let d = diff_front(&a, &b);
    assert!(d.flags.contains(&"creator-lost".to_string()), "{d:?}");
    assert!(
      d.notes.iter().any(|n| n.contains("creator lost \"Bob\"")),
      "{d:?}"
    );
  }

  #[test]
  fn a_one_character_contact_link_is_flagged() {
    // A typewriter address split by a `$\sim$` (1601.02576 in 64zc r1): with an element child that is not a
    // typewriter run and no bare text, the XSLT links the first text inside, one character.
    let a = front(&format!(
      r#"{HEAD}<creator role="author"><personname>A</personname>
      <contact role="email">h∼koesters@x.edu</contact></creator></document>"#
    ));
    let b = front(&format!(
      r#"{HEAD}<creator role="author"><personname>A</personname>
      <contact role="email"><text font="typewriter">h</text><Math tex="\sim"><XMath><XMTok>∼</XMTok></XMath></Math><text font="typewriter">koesters@x.edu</text></contact></creator></document>"#
    ));
    assert_eq!(
      a.creators[0].contacts[0].link.as_deref(),
      Some("mailto:h∼koesters@x.edu")
    );
    assert_eq!(b.creators[0].contacts[0].link.as_deref(), Some("mailto:h"));
    let d = diff_front(&a, &b);
    for flag in [
      "contact-split",
      "contact-link-changed",
      "contact-link-fragment",
      "creator-changed",
    ] {
      assert!(d.flags.contains(&flag.to_string()), "{flag}: {d:?}");
    }
  }

  #[test]
  fn an_orcid_links_unless_a_ref_is_its_child() {
    let f = front(&format!(
      r#"{HEAD}<creator><personname>A</personname><contact role="orcid"><text><ref href="x">0000-1</ref></text></contact><contact role="orcid"><ref href="y">0000-2</ref></contact><contact role="email"><text><ref href="mailto:z">z</ref></text></contact></creator></document>"#
    ));
    let c = &f.creators[0].contacts;
    assert_eq!(c[0].link.as_deref(), Some("https://orcid.org/0000-1"));
    assert_eq!(c[1].link, None);
    assert_eq!(c[2].link, None);
  }

  #[test]
  fn typewriter_runs_around_bare_text_link_their_whole_text() {
    // The current XSLT's first branch: every element child a typewriter run.
    let f = front(&format!(
      r#"{HEAD}<creator><personname>A</personname><contact role="url"><text font="typewriter">http://x.edu/</text>∼<text font="typewriter">ann</text></contact></creator></document>"#
    ));
    assert_eq!(
      f.creators[0].contacts[0].link.as_deref(),
      Some("http://x.edu/∼ann")
    );
    assert!(f.creators[0].contacts[0].split);
  }

  #[test]
  fn all_typewriter_runs_link_their_whole_text() {
    let f = front(&format!(
      r#"{HEAD}<creator><personname>A</personname><contact role="email"><text font="typewriter">a_b</text><text font="typewriter">@c.d</text></contact></creator></document>"#
    ));
    assert_eq!(
      f.creators[0].contacts[0].link.as_deref(),
      Some("mailto:a_b@c.d")
    );
  }
}
