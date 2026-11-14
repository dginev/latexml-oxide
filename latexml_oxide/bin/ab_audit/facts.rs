//! What one conversion produced, read from its log and its XML: the per-document extractor shared by
//! `compare` (two trees) and `audit` (one tree).
//!
//! The log is read ANSI-stripped, so a coloured `\x1b[31mError:` line counts (the bug that once hid
//! 2002.05958's 654 errors as "fixed", CLAUDE.md). The XML is parsed by `roxmltree`, a pure-Rust reader,
//! so each worker thread owns its parser (rust-libxml is not shared across threads in this project).

use std::collections::BTreeMap;

use roxmltree::{Document, Node, NodeType, ParsingOptions};
use rustc_hash::FxHashSet;

use crate::{
  canon::{attr_value, volatile},
  front::{Front, read_front},
};

/// The LaTeXML namespace: its elements are named by their local name alone.
pub const LTX_NS: &str = "http://dlmf.nist.gov/LaTeXML";
const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XHTML_NS: &str = "http://www.w3.org/1999/xhtml";
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// Inline wrappers whose text belongs to the element around them: a font change (`text`), an emphasis
/// around an address does not move its words to another class. A `ref`, `cite` or `bibref` is not one: its
/// text is generated (a reference number, a citation label), so it is a class of its own and a `\ref` now
/// printing `3` is not new prose.
const TRANSPARENT: &[&str] = &["text", "emph", "anchor", "sup", "sub", "del", "glossaryref"];

/// Attributes whose values are content: a link target, a label, a reference number, a formula's source,
/// an image path. Their text is compared like element text, as class `<element>@<attribute>`.
pub const TEXT_ATTRS: &[&str] = &[
  "href",
  "content",
  "alt",
  "title",
  "description",
  "labels",
  "refnum",
  "frefnum",
  "rrefnum",
  "tex",
  "graphic",
  "key",
  // Rendered by the XSLT around the element's content (LaTeXML-structure-xhtml.xsl:146-240: a contact's
  // `name="E-mail: "`, a list item's or a theorem's `before`/`after` text).
  "name",
  "before",
  "after",
];

/// Where a child element that is not a wrapper interrupts a segment (`see <Math/>`): a text segment keeps
/// this object-replacement character there, so a space lost next to the child shows (`see<Math/>`).
pub const BOUNDARY: char = '\u{FFFC}';

/// Attributes that are ids or references to ids: compared by the canonical diff's id mapping, not by value.
pub const ID_ATTRS: &[&str] = &[
  "xml:id",
  "id",
  "fragid",
  "idref",
  "aboutidref",
  "resourceidref",
];

/// SVG geometry, and a picture's size: numbers that move by rounding when anything upstream moves. Compared
/// in document order with a [`tolerance`], a change beyond it is `geometry-changed`.
const SVG_GEOMETRY: &[&str] = &[
  "d",
  "transform",
  "viewBox",
  "x",
  "y",
  "width",
  "height",
  "x1",
  "y1",
  "x2",
  "y2",
  "cx",
  "cy",
  "r",
  "rx",
  "ry",
  "points",
  "dx",
  "dy",
  "stroke-width",
  "font-size",
  "stroke-dasharray",
  "stroke-dashoffset",
  "offset",
  "opacity",
  "fill-opacity",
  "stroke-opacity",
];

/// The largest difference between two geometry or text numbers that is not a change: 0.05, relative for
/// small magnitudes (2 %, at least 0.005), so a matrix entry 0.01 → 0.05 or a stroke 0.04 → 0 is a change.
pub fn tolerance(p: f64, q: f64) -> f64 { (0.02 * p.abs().max(q.abs())).clamp(0.005, 0.05) }

/// Whether `qname@attr` is geometry.
pub fn is_geometry(qname: &str, attr: &str) -> bool {
  (qname.starts_with("svg:") && SVG_GEOMETRY.contains(&attr))
    || (qname == "picture" && matches!(attr, "width" | "height"))
}

/// The numbers in a value, and the value with each number replaced by `N`.
pub fn numbers(value: &str) -> (Vec<f64>, String) {
  let (nums, _, skeleton) = numbers_with_digits(value);
  (nums, skeleton)
}

/// [`numbers`], with each number's count of fractional digits (`3.80254pt` 5, `2019` 0).
pub fn numbers_with_digits(value: &str) -> (Vec<f64>, Vec<usize>, String) {
  let (mut nums, mut digits, mut skeleton) = (Vec::new(), Vec::new(), String::new());
  let bytes = value.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let start = i;
    if matches!(bytes[i], b'-' | b'+')
      && bytes
        .get(i + 1)
        .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
    {
      i += 1;
    }
    let digits_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
      i += 1;
    }
    if i > digits_start
      && let Ok(n) = value[start..i].parse::<f64>()
    {
      if i < bytes.len() && matches!(bytes[i], b'e' | b'E') {
        // An exponent: part of the number.
        let mut j = i + 1;
        if j < bytes.len() && matches!(bytes[j], b'-' | b'+') {
          j += 1;
        }
        let exp_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
          j += 1;
        }
        if j > exp_start
          && let Ok(m) = value[start..j].parse::<f64>()
        {
          nums.push(m);
          digits.push(fraction_digits(&value[start..i]));
          skeleton.push('N');
          i = j;
          continue;
        }
      }
      nums.push(n);
      digits.push(fraction_digits(&value[start..i]));
      skeleton.push('N');
      continue;
    }
    i = start;
    let c = value[i..].chars().next().unwrap_or(' ');
    skeleton.push(c);
    i += c.len_utf8();
  }
  (nums, digits, skeleton)
}

/// The digits after the `.` of a number as printed.
fn fraction_digits(number: &str) -> usize { number.split_once('.').map_or(0, |(_, f)| f.len()) }

/// A value that refers to an id of this document (`#S1.p1`, `url(#pgfcp1)`) with the id replaced by `@`:
/// ids renumber between runs, and the canonical diff checks where references point.
pub fn ref_value(value: &str, ids: &FxHashSet<&str>) -> String {
  if let Some(id) = value.strip_prefix('#')
    && ids.contains(id)
  {
    return "#@".to_string();
  }
  if let Some(id) = value
    .strip_prefix("url(#")
    .and_then(|v| v.strip_suffix(')'))
    && ids.contains(id)
  {
    return "url(#@)".to_string();
  }
  attr_value(value)
}

/// The ids of a document: `xml:id` and a plain `id` (SVG).
pub fn document_ids<'a>(doc: &'a Document<'_>) -> FxHashSet<&'a str> {
  doc
    .descendants()
    .filter(Node::is_element)
    .flat_map(|n| n.attributes())
    .filter(|a| a.name() == "id")
    .map(|a| a.value())
    .collect()
}

/// The diagnostics of one log.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LogFacts {
  /// The cortex status of the last `Status:conversion:N` line, when it reads 0..=3: 3 fatal, 2 error,
  /// lower OK.
  pub status:           Option<u8>,
  /// The last `Status:conversion:` line holds something other than 0..=3 (`x7`, `300`, nothing).
  pub status_bad:       bool,
  /// Line-anchored counts, as cortex and `grep -c '^Error:'` count them.
  pub errors:           u32,
  pub fatals:           u32,
  pub warnings:         u32,
  /// `Level:category:` found inside a line that does not start with a diagnostic (a message printed on
  /// the same line as a `(Loading …` progress mark): not in the totals, flagged on their own.
  pub midline_errors:   u32,
  pub midline_fatals:   u32,
  pub midline_warnings: u32,
  /// Counts by `Level:category:object`, the first word of each line-anchored diagnostic.
  pub classes:          BTreeMap<String, u32>,
}

impl LogFacts {
  /// Whether the logged diagnostics (line-anchored and mid-line) disagree with the cortex status: status
  /// 2 with no Error, status 3 with no Fatal, or a lower status than the Errors/Fatals imply. The signal of
  /// a converter that recovered silently, or counted without logging. `false` without a readable status
  /// (that is `no-status` or `status-unparsable`, flagged on their own).
  pub fn status_mismatch(&self) -> bool {
    let (errors, fatals) = (
      self.errors + self.midline_errors,
      self.fatals + self.midline_fatals,
    );
    match self.status {
      Some(3) => fatals == 0,
      Some(2) => errors == 0 || fatals > 0,
      Some(_) => errors > 0 || fatals > 0,
      None => false,
    }
  }
}

/// The level of a `Level:category:` diagnostic starting at `text`'s beginning: `Error`, `Fatal` or
/// `Warning`, then a non-empty category without spaces or colons, then `:`. `Error: my own typeout` (free
/// text, no category) is not one.
fn diagnostic_level(text: &str) -> Option<usize> {
  let (level, rest) = ["Error:", "Fatal:", "Warning:"]
    .iter()
    .enumerate()
    .find_map(|(i, l)| text.strip_prefix(l).map(|r| (i, r)))?;
  let category_len = rest
    .find(|c: char| c == ':' || c.is_whitespace())
    .unwrap_or(rest.len());
  (category_len > 0 && rest[category_len..].starts_with(':')).then_some(level)
}

/// Strip ANSI CSI sequences (`ESC [ … letter`), the colour the logger may write around a level.
pub fn strip_ansi(line: &str) -> std::borrow::Cow<'_, str> {
  if !line.contains('\x1b') {
    return std::borrow::Cow::Borrowed(line);
  }
  let mut out = String::with_capacity(line.len());
  let mut chars = line.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\x1b' && chars.peek() == Some(&'[') {
      chars.next();
      for d in chars.by_ref() {
        if d.is_ascii_alphabetic() {
          break;
        }
      }
    } else {
      out.push(c);
    }
  }
  std::borrow::Cow::Owned(out)
}

/// Read a conversion log: diagnostics by class and the closing status.
pub fn read_log(text: &str) -> LogFacts {
  let mut facts = LogFacts::default();
  for raw in text.lines() {
    let line = strip_ansi(raw);
    let line = line.as_ref();
    if let Some(rest) = line.strip_prefix("Status:conversion:") {
      let status = rest.trim().parse::<u8>().ok().filter(|s| *s <= 3);
      (facts.status, facts.status_bad) = (status, status.is_none());
      continue;
    }
    let anchored = if line.starts_with("Error:") {
      Some(&mut facts.errors)
    } else if line.starts_with("Fatal:") {
      Some(&mut facts.fatals)
    } else if line.starts_with("Warning:") {
      Some(&mut facts.warnings)
    } else {
      None
    };
    if let Some(level) = anchored {
      *level += 1;
      let class = line.split([' ', '\t']).next().unwrap_or(line);
      *facts.classes.entry(class.to_string()).or_default() += 1;
    }
    // Every other `Level:category:` on the line, after an anchored head too (`Error:… (Loading b.sty
    // Fatal:too_many_errors:100 …`): not in the totals, counted apart.
    for (i, _) in line
      .char_indices()
      .skip(1)
      .filter(|(_, c)| matches!(c, 'E' | 'F' | 'W'))
    {
      match diagnostic_level(&line[i..]) {
        Some(0) => facts.midline_errors += 1,
        Some(1) => facts.midline_fatals += 1,
        Some(_) => facts.midline_warnings += 1,
        None => {},
      }
    }
  }
  facts
}

/// Collapse each run of ASCII whitespace to one space and trim the ends. A lost space between two words
/// still shows; a no-break space (`~`) is not whitespace here, so a space turned into one shows too.
pub fn collapse(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  for word in text.split_ascii_whitespace() {
    if !out.is_empty() {
      out.push(' ');
    }
    out.push_str(word);
  }
  out
}

/// An element's name: its local name in the LaTeXML namespace, a short prefix for MathML, SVG and XHTML,
/// `{namespace}local` otherwise.
pub fn qname(node: Node<'_, '_>) -> String {
  let local = node.tag_name().name();
  match node.tag_name().namespace() {
    None | Some(LTX_NS) => local.to_string(),
    Some(MATHML_NS) => format!("m:{local}"),
    Some(SVG_NS) => format!("svg:{local}"),
    Some(XHTML_NS) => format!("xhtml:{local}"),
    Some(ns) => format!("{{{ns}}}{local}"),
  }
}

/// An attribute's name: `xml:id`, its local name without a namespace, `{namespace}local` otherwise.
pub fn attr_name(attr: &roxmltree::Attribute<'_, '_>) -> String {
  match attr.namespace() {
    None => attr.name().to_string(),
    Some(XML_NS) => format!("xml:{}", attr.name()),
    Some(ns) => format!("{{{ns}}}{}", attr.name()),
  }
}

/// Whether `node` is the LaTeXML element `local`.
pub fn is_ltx(node: Node<'_, '_>, local: &str) -> bool {
  node.is_element()
    && node.tag_name().name() == local
    && matches!(node.tag_name().namespace(), None | Some(LTX_NS))
}

/// The collapsed text of every text node under `node`.
pub fn text_of(node: Node<'_, '_>) -> String {
  let mut raw = String::new();
  for d in node.descendants() {
    if d.is_text()
      && let Some(t) = d.text()
    {
      raw.push_str(t);
    }
  }
  collapse(&raw)
}

/// The content facts of one XML document.
#[derive(Debug, Default, Clone)]
pub struct XmlFacts {
  /// Element counts by name.
  pub elements: BTreeMap<String, u32>,
  /// Text by class, as the document-ordered list of its non-empty collapsed segments. A class is the
  /// nearest enclosing element that is not a [`TRANSPARENT`] wrapper (`p`, `title`, `contact`, `XMTok` …),
  /// one segment per element instance; attribute text is class `<element>@<attribute>`, one segment
  /// per value.
  pub text:     BTreeMap<String, Vec<String>>,
  /// Every attribute value that is neither an id, nor compared as text ([`TEXT_ATTRS`]), nor geometry, as
  /// a multiset of `qname@attr=value` (host paths, heap addresses, pgf names and references to this
  /// document's ids normalised).
  pub attrs:    BTreeMap<String, u32>,
  /// Geometry values by `qname@attr`, in document order.
  pub geometry: BTreeMap<String, Vec<String>>,
  /// The frontmatter tuples.
  pub front:    Front,
  /// Whitespace-separated tokens of element text, for the audit's size column.
  pub words:    usize,
}

/// Parse XML bytes, or say why they cannot be read: empty, not UTF-8, or not well-formed.
pub fn parse(bytes: &[u8]) -> Result<Document<'_>, String> {
  if bytes.iter().all(u8::is_ascii_whitespace) {
    return Err("empty file".to_string());
  }
  let text = std::str::from_utf8(bytes).map_err(|e| format!("not UTF-8: {e}"))?;
  let options = ParsingOptions {
    allow_dtd: true,
    ..ParsingOptions::default()
  };
  Document::parse_with_options(text, options).map_err(|e| format!("not well-formed: {e}"))
}

/// Read the content facts of a parsed document.
pub fn read_xml(doc: &Document<'_>) -> XmlFacts {
  let mut facts = XmlFacts::default();
  let ids = document_ids(doc);
  // Open segments, innermost last: (class, index of its segment in `facts.text[class]`).
  let mut open: Vec<(String, usize)> = Vec::new();
  // Explicit-stack walk (deep documents must not overflow the thread stack): `true` = leaving.
  let mut stack: Vec<(Node<'_, '_>, bool)> = vec![(doc.root(), false)];
  while let Some((node, leaving)) = stack.pop() {
    match node.node_type() {
      NodeType::Root => push_children(&mut stack, node),
      NodeType::Element => {
        let name = qname(node);
        let transparent = matches!(node.tag_name().namespace(), None | Some(LTX_NS))
          && TRANSPARENT.contains(&node.tag_name().name());
        if leaving {
          if !transparent {
            open.pop();
          }
          continue;
        }
        *facts.elements.entry(name.clone()).or_default() += 1;
        for attr in node.attributes() {
          let attr_key = attr_name(&attr);
          // A math token's `name` is its semantic name, not rendered text (compared as an attribute).
          let math_name =
            name.starts_with("XM") && matches!(attr_key.as_str(), "name" | "before" | "after");
          if ID_ATTRS.contains(&attr_key.as_str()) {
          } else if TEXT_ATTRS.contains(&attr_key.as_str()) && !math_name {
            // Compared as text only, so one lost `name="E-mail: "` is one flag.
            let value = collapse(&ref_value(attr.value(), &ids));
            if !value.is_empty() {
              facts
                .text
                .entry(format!("{name}@{attr_key}"))
                .or_default()
                .push(value);
            }
          } else if is_geometry(&name, &attr_key) {
            facts
              .geometry
              .entry(format!("{name}@{attr_key}"))
              .or_default()
              .push(attr.value().to_string());
          } else {
            *facts
              .attrs
              .entry(format!(
                "{name}@{attr_key}={}",
                ref_value(attr.value(), &ids)
              ))
              .or_default() += 1;
          }
        }
        if !transparent {
          if let Some((class, index)) = open.last()
            && let Some(parent) = facts.text.get_mut(class).and_then(|s| s.get_mut(*index))
          {
            parent.push(BOUNDARY);
          }
          let segments = facts.text.entry(name.clone()).or_default();
          segments.push(String::new());
          open.push((name, segments.len() - 1));
        }
        stack.push((node, true));
        push_children(&mut stack, node);
      },
      NodeType::Text => {
        if let Some(t) = node.text() {
          let (class, index) = match open.last() {
            Some((class, index)) => (class.as_str(), *index),
            None => ("#document", 0),
          };
          let segments = facts.text.entry(class.to_string()).or_default();
          if segments.is_empty() {
            segments.push(String::new());
          }
          segments[index].push_str(t);
        }
      },
      NodeType::Comment | NodeType::PI => {},
    }
  }
  for segments in facts.text.values_mut() {
    for segment in segments.iter_mut() {
      *segment = volatile(&collapse(segment));
    }
    // A segment of boundaries alone (a `para` around its `p`s) holds no text.
    segments.retain(|s| s.chars().any(|c| c != ' ' && c != BOUNDARY));
  }
  facts.text.retain(|_, segments| !segments.is_empty());
  facts.words = facts
    .text
    .iter()
    .filter(|(class, _)| !class.contains('@'))
    .flat_map(|(_, segments)| segments.iter())
    .flat_map(|s| s.split(' '))
    .filter(|t| *t != BOUNDARY.to_string())
    .count();
  facts.front = read_front(doc);
  facts
}

/// Push `node`'s children so that they pop in document order.
fn push_children<'a, 'i>(stack: &mut Vec<(Node<'a, 'i>, bool)>, node: Node<'a, 'i>) {
  let mark = stack.len();
  stack.extend(node.children().map(|c| (c, false)));
  stack[mark..].reverse();
}

#[cfg(test)]
mod tests {
  use super::*;

  fn facts(xml: &str) -> XmlFacts { read_xml(&parse(xml.as_bytes()).expect("well-formed")) }

  #[test]
  fn an_ansi_coloured_error_line_counts() {
    let log = "\x1b[31mError:\x1b[0mundefined:\\foo The token is not defined.\n\
               \tat a; line 1\nWarning:missing_file:x.sty gone\n\x1b[1;31mFatal:\x1b[0mtimeout:convert slow\n\
               Status:conversion:3\n";
    let facts = read_log(log);
    assert_eq!((facts.errors, facts.warnings, facts.fatals), (1, 1, 1));
    assert_eq!(facts.status, Some(3));
    assert_eq!(facts.classes.get("Error:undefined:\\foo"), Some(&1));
    assert_eq!(facts.classes.get("Fatal:timeout:convert"), Some(&1));
  }

  #[test]
  fn a_status_the_diagnostics_do_not_explain_is_a_mismatch() {
    assert!(
      read_log("Status:conversion:2\n").status_mismatch(),
      "status 2, no Error line"
    );
    assert!(
      read_log("Error:x:y z\nStatus:conversion:1\n").status_mismatch(),
      "an Error under status 1"
    );
    assert!(
      read_log("Status:conversion:3\n").status_mismatch(),
      "status 3, no Fatal line"
    );
    assert!(!read_log("Error:x:y z\nStatus:conversion:2\n").status_mismatch());
    assert!(!read_log("Error:x:y z\nFatal:a:b c\nStatus:conversion:3\n").status_mismatch());
    assert!(!read_log("Warning:x:y z\nStatus:conversion:1\n").status_mismatch());
    assert!(!read_log("no status\n").status_mismatch());
  }

  #[test]
  fn a_status_outside_0_to_3_is_unparsable_not_clean() {
    for log in [
      "Status:conversion:x7\n",
      "Status:conversion:\n",
      "Status:conversion:300\n",
    ] {
      let f = read_log(log);
      assert_eq!((f.status, f.status_bad), (None, true), "{log:?}");
    }
    let f = read_log("Status:conversion:x7\nStatus:conversion:2\nError:a:b c\n");
    assert_eq!((f.status, f.status_bad), (Some(2), false));
  }

  #[test]
  fn a_mid_line_diagnostic_is_counted_apart_and_free_text_is_not() {
    let f = read_log("(Loading foo.sty... Error:undefined:\\x bad\nStatus:conversion:0\n");
    assert_eq!((f.errors, f.midline_errors), (0, 1));
    assert!(f.status_mismatch());
    let f = read_log("see Error: my own typeout\nStatus:conversion:0\n");
    assert!(f.midline_errors == 0 && !f.status_mismatch());
    // A line-anchored free-text `Error:` still counts in the totals (cortex parity).
    assert_eq!(read_log("Error: my own typeout\n").errors, 1);
  }

  #[test]
  fn every_diagnostic_on_a_line_counts() {
    // /tmp/rev_p0r4 A5/B5 e: a Fatal after an anchored Error; w: an Error after a mid-line Warning.
    let f = read_log(
      "Error:undefined:\\a x (Loading b.sty Fatal:too_many_errors:100 boom\nStatus:conversion:2\n",
    );
    assert_eq!((f.errors, f.midline_fatals), (1, 1));
    let f = read_log(
      "(Loading a.sty... Warning:x:y foo Error:undefined:\\bar bad)\nStatus:conversion:1\n",
    );
    assert_eq!((f.midline_warnings, f.midline_errors), (1, 1));
    // Lower case is not a level.
    assert_eq!(read_log("(Loading a.sty... fatal:x:y)\n").midline_fatals, 0);
  }

  #[test]
  fn a_log_without_status_reads_none() {
    assert_eq!(read_log("Error:x:y z\n").status, None);
  }

  #[test]
  fn text_is_classed_by_the_nearest_non_wrapper() {
    let f = facts(
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><para><p>Hello <text font="bold">big</text>
      world<note role="footnote">aside</note>.</p></para><graphics graphic="a.png"/></document>"#,
    );
    assert_eq!(f.text["p"], vec!["Hello big world\u{FFFC}."]);
    assert_eq!(f.text["note"], vec!["aside"]);
    assert_eq!(f.text["graphics@graphic"], vec!["a.png"]);
    assert_eq!(f.elements["graphics"], 1);
    assert_eq!(f.elements["text"], 1);
  }

  #[test]
  fn a_space_before_a_non_wrapper_child_is_kept_in_the_segment() {
    let spaced = facts(
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><p>see <Math tex="a"/> and</p></document>"#,
    );
    let tight = facts(
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><p>see<Math tex="a"/> and</p></document>"#,
    );
    assert_eq!(spaced.text["p"], vec!["see \u{FFFC} and"]);
    assert_eq!(tight.text["p"], vec!["see\u{FFFC} and"]);
    assert_eq!(spaced.words, 2);
  }

  #[test]
  fn every_non_id_attribute_value_is_counted() {
    let f = facts(
      r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"><p xml:id="p1" class="ltx_centering">x<ref idref="p1"/></p></document>"#,
    );
    assert_eq!(f.attrs.get("p@class=ltx_centering"), Some(&1));
    assert!(
      f.attrs
        .keys()
        .all(|k| !k.contains("xml:id") && !k.contains("idref")),
      "{:?}",
      f.attrs
    );
  }

  #[test]
  fn collapse_keeps_single_characters_and_no_break_spaces() {
    assert_eq!(collapse("  a \n b\u{a0}c  "), "a b\u{a0}c");
  }

  #[test]
  fn unparsable_xml_is_an_error_not_an_empty_document() {
    assert!(parse(b"<document><p>unclosed</document>").is_err());
    assert!(parse(b"").is_err());
    assert!(parse(b"\xff\xfe<a/>").is_err());
  }
}
