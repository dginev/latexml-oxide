//! The canonical XML diff: each document as a line per start tag (attributes sorted), text run, end tag,
//! comment and processing instruction. Ids are MAPPED, not stripped: an `xml:id`/`fragid` becomes `#`
//! and an `idref` `@` in the compared line, the real values ride beside it, A's ids are paired with B's
//! along the diff's equal runs, and every reference in an equal line must still point at the paired
//! target. No wrapper is stripped: a font run added around a contact is a changed line.
//!
//! Volatile host detail is normalised in values: a `/tmp/<dir>` path (the graphicspath PI of an archive
//! unpacked to a temporary directory) and a Rust primitive's heap address in a `\meaning`.

use std::{
  hash::{Hash, Hasher},
  time::{Duration, Instant},
};

use roxmltree::{Document, Node, NodeType};
use rustc_hash::{FxHashMap, FxHasher};
use similar::{Algorithm, DiffOp, capture_diff_slices_deadline};

use crate::facts::{attr_name, collapse, qname};

/// One document as canonical lines.
#[derive(Debug, Default)]
pub struct Canon {
  pub lines: Vec<String>,
  /// The line's own id (`xml:id`), if any.
  pub ids:   Vec<Option<String>>,
  /// The ids the line refers to (`idref`).
  pub refs:  Vec<Vec<String>>,
}

/// Replace `/tmp/<name>` by `/tmp/*` and a `0x…` heap address after `CODE(`, `addr: ` or
/// `DynMetadata(` by `0x`.
pub fn volatile(value: &str) -> String {
  if !value.contains("/tmp/") && !value.contains("0x") {
    return value.to_string();
  }
  let mut out = String::with_capacity(value.len());
  let mut rest = value;
  while !rest.is_empty() {
    if let Some(after) = rest.strip_prefix("/tmp/") {
      out.push_str("/tmp/*");
      let end = after
        .find(|c: char| c == '/' || c == '"' || c.is_whitespace())
        .unwrap_or(after.len());
      rest = &after[end..];
      continue;
    }
    let addr_lead = ["CODE(", "addr: ", "DynMetadata("]
      .iter()
      .find(|p| rest.starts_with(**p));
    if let Some(lead) = addr_lead
      && let Some(hex) = rest[lead.len()..].strip_prefix("0x")
    {
      out.push_str(lead);
      out.push_str("0x");
      let end = hex
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(hex.len());
      rest = &hex[end..];
      continue;
    }
    let c = rest.chars().next().unwrap_or(' ');
    out.push(c);
    rest = &rest[c.len_utf8()..];
  }
  out
}

/// An id-shaped attribute value naming a pgf object numbered per run (`pgfcp12`, `#pgfpat3`,
/// `url(#pgfcp1)`), with its number replaced by `N`; `None` for anything else, so text such as `pgfplots1`
/// keeps its digits.
pub fn pgf_id(value: &str) -> Option<String> {
  let (lead, inner, tail) = if let Some(v) = value
    .strip_prefix("url(#")
    .and_then(|v| v.strip_suffix(')'))
  {
    ("url(#", v, ")")
  } else if let Some(v) = value.strip_prefix('#') {
    ("#", v, "")
  } else {
    ("", value, "")
  };
  let after = inner.strip_prefix("pgf")?;
  let letters = after
    .find(|c: char| !c.is_ascii_lowercase())
    .unwrap_or(after.len());
  let digits = &after[letters..];
  (letters > 0 && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
    .then(|| format!("{lead}pgf{}N{tail}", &after[..letters]))
}

/// An attribute value with volatile host detail and per-run pgf object names normalised.
pub fn attr_value(value: &str) -> String { pgf_id(value).unwrap_or_else(|| volatile(value)) }

/// Each run of ASCII whitespace as one space, ends kept: a text run's leading or trailing space is a
/// space between words (`foo <b>bar</b>`), so it must not be trimmed (64b: a guard's trim hid a lost space).
pub fn squeeze(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut in_space = false;
  for c in text.chars() {
    if c.is_ascii_whitespace() {
      if !in_space {
        out.push(' ');
      }
      in_space = true;
    } else {
      out.push(c);
      in_space = false;
    }
  }
  out
}

/// Canonical lines of a parsed document.
pub fn canonical(doc: &Document<'_>) -> Canon {
  let mut canon = Canon::default();
  let ids = crate::facts::document_ids(doc);
  let mut push = |line: String, id: Option<String>, refs: Vec<String>| {
    canon.lines.push(line);
    canon.ids.push(id);
    canon.refs.push(refs);
  };
  let mut stack: Vec<(Node<'_, '_>, bool)> = vec![(doc.root(), false)];
  while let Some((node, leaving)) = stack.pop() {
    match node.node_type() {
      NodeType::Root | NodeType::Element if leaving => {
        if node.is_element() {
          push(format!("</{}>", qname(node)), None, Vec::new());
        }
      },
      NodeType::Root | NodeType::Element => {
        if node.is_element() {
          let mut attrs: Vec<(String, String)> = Vec::new();
          let (mut id, mut refs) = (None, Vec::new());
          for a in node.attributes() {
            let name = attr_name(&a);
            let value = a.value();
            let in_doc = |v: &str| ids.contains(v).then(|| v.to_string());
            // A `#id` or `url(#id)` naming an id of this document is a reference like `idref`.
            let local_ref = value
              .strip_prefix('#')
              .or_else(|| {
                value
                  .strip_prefix("url(#")
                  .and_then(|v| v.strip_suffix(')'))
              })
              .and_then(in_doc);
            let shown = match name.as_str() {
              "xml:id" | "id" => {
                id = Some(value.to_string());
                "#".to_string()
              },
              "fragid" => "#".to_string(),
              "idref" | "aboutidref" | "resourceidref" => {
                refs.push(value.to_string());
                "@".to_string()
              },
              _ if local_ref.is_some() => {
                refs.extend(local_ref);
                "@".to_string()
              },
              // Geometry numbers are compared with a tolerance by the facts (`geometry-changed`); here
              // only the shape of the value counts.
              _ if crate::facts::is_geometry(&qname(node), &name) => crate::facts::numbers(value).1,
              _ => attr_value(value),
            };
            attrs.push((name, shown));
          }
          attrs.sort();
          let mut line = format!("<{}", qname(node));
          for (name, value) in attrs {
            line.push_str(&format!(" {name}={value:?}"));
          }
          line.push('>');
          push(line, id, refs);
        }
        stack.push((node, true));
        let mark = stack.len();
        stack.extend(node.children().map(|c| (c, false)));
        stack[mark..].reverse();
      },
      NodeType::Text => {
        let raw = node.text().unwrap_or("");
        // Indentation between block elements carries a newline; a space inside mixed content does not.
        if raw.trim_ascii().is_empty() && raw.contains('\n') {
          continue;
        }
        push(format!("\"{}\"", volatile(&squeeze(raw))), None, Vec::new());
      },
      NodeType::Comment => push(
        format!("<!--{}-->", collapse(node.text().unwrap_or(""))),
        None,
        Vec::new(),
      ),
      NodeType::PI => {
        if let Some(pi) = node.pi() {
          push(
            format!("<?{} {}?>", pi.target, volatile(pi.value.unwrap_or(""))),
            None,
            Vec::new(),
          );
        }
      },
    }
  }
  canon
}

/// The canonical difference of two documents.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CanonDiff {
  pub deleted:         usize,
  pub inserted:        usize,
  /// Equal lines whose ids differ (renumbered), and references in equal lines that point at a target not
  /// paired with A's target.
  pub ids_renamed:     usize,
  pub refs_retargeted: usize,
  /// Whether the diff hit its deadline (an approximate, still non-empty, difference).
  pub approximate:     bool,
  /// Hunks for the detail file: `-`/`+`/` ` prefixed lines.
  pub hunks:           Vec<String>,
}

fn line_hash(line: &str) -> u64 {
  let mut h = FxHasher::default();
  line.hash(&mut h);
  h.finish()
}

/// Diff two canonical documents (Myers, 10 s deadline), map ids along the equal runs and check references.
pub fn diff(a: &Canon, b: &Canon, max_hunks: usize) -> CanonDiff {
  let ha: Vec<u64> = a.lines.iter().map(|l| line_hash(l)).collect();
  let hb: Vec<u64> = b.lines.iter().map(|l| line_hash(l)).collect();
  let deadline = Instant::now() + Duration::from_secs(10);
  let ops = capture_diff_slices_deadline(Algorithm::Myers, &ha, &hb, Some(deadline));
  let mut out = CanonDiff {
    approximate: Instant::now() > deadline,
    ..CanonDiff::default()
  };
  let mut map: FxHashMap<&str, &str> = FxHashMap::default();
  for op in &ops {
    match *op {
      DiffOp::Equal { old_index, new_index, len } => {
        for k in 0..len {
          if let (Some(x), Some(y)) = (&a.ids[old_index + k], &b.ids[new_index + k]) {
            map.insert(x, y);
            if x != y {
              out.ids_renamed += 1;
            }
          }
        }
      },
      DiffOp::Delete { old_len, .. } => out.deleted += old_len,
      DiffOp::Insert { new_len, .. } => out.inserted += new_len,
      DiffOp::Replace { old_len, new_len, .. } => {
        out.deleted += old_len;
        out.inserted += new_len;
      },
    }
  }
  for op in &ops {
    if let DiffOp::Equal { old_index, new_index, len } = *op {
      for k in 0..len {
        for (x, y) in a.refs[old_index + k].iter().zip(&b.refs[new_index + k]) {
          if let Some(target) = map.get(x.as_str())
            && *target != y.as_str()
          {
            out.refs_retargeted += 1;
          }
        }
      }
    }
  }
  // Hunks with 2 lines of context, at most `max_hunks`, each at most 40 changed lines.
  let ops_len = ops.len();
  for (n, op) in ops.iter().enumerate() {
    if out.hunks.len() >= max_hunks {
      break;
    }
    let (oi, ol, ni, nl) = match *op {
      DiffOp::Equal { .. } => continue,
      DiffOp::Delete {
        old_index, old_len, new_index, ..
      } => (old_index, old_len, new_index, 0),
      DiffOp::Insert {
        old_index, new_index, new_len, ..
      } => (old_index, 0, new_index, new_len),
      DiffOp::Replace {
        old_index,
        old_len,
        new_index,
        new_len,
      } => (old_index, old_len, new_index, new_len),
    };
    let mut hunk = format!(
      "@@ A line {} B line {} (op {}/{ops_len})\n",
      oi + 1,
      ni + 1,
      n + 1
    );
    for line in &a.lines[oi.saturating_sub(2)..oi] {
      hunk.push_str(&format!("  {line}\n"));
    }
    for line in a.lines[oi..oi + ol].iter().take(40) {
      hunk.push_str(&format!("- {line}\n"));
    }
    if ol > 40 {
      hunk.push_str(&format!("- … {} more\n", ol - 40));
    }
    for line in b.lines[ni..ni + nl].iter().take(40) {
      hunk.push_str(&format!("+ {line}\n"));
    }
    if nl > 40 {
      hunk.push_str(&format!("+ … {} more\n", nl - 40));
    }
    out.hunks.push(hunk);
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::facts::parse;

  fn canon(xml: &str) -> Canon { canonical(&parse(xml.as_bytes()).expect("well-formed")) }

  #[test]
  fn renumbered_ids_with_consistent_references_are_not_a_change() {
    let a = canon(r#"<d><p xml:id="a1">x<ref idref="a1"/></p></d>"#);
    let b = canon(r#"<d><p xml:id="b9">x<ref idref="b9"/></p></d>"#);
    let d = diff(&a, &b, 5);
    assert_eq!(
      (d.deleted, d.inserted, d.ids_renamed, d.refs_retargeted),
      (0, 0, 1, 0)
    );
  }

  #[test]
  fn a_reference_moved_to_another_target_shows() {
    let a = canon(r#"<d><p xml:id="s1">x</p><p xml:id="s2">y</p><ref idref="s1"/></d>"#);
    let b = canon(r#"<d><p xml:id="s1">x</p><p xml:id="s2">y</p><ref idref="s2"/></d>"#);
    assert_eq!(diff(&a, &b, 5).refs_retargeted, 1);
  }

  #[test]
  fn an_added_wrapper_is_a_changed_line_and_tmp_paths_are_not() {
    let a = canon(r#"<?latexml graphicspath="/tmp/abc123/"?><d><c>ann</c></d>"#);
    let b = canon(
      r#"<?latexml graphicspath="/tmp/zz9/"?><d><c><text font="typewriter">ann</text></c></d>"#,
    );
    let d = diff(&a, &b, 5);
    assert_eq!(a.lines[0], b.lines[0]);
    assert_eq!((d.deleted, d.inserted), (0, 2));
  }

  #[test]
  fn renumbered_svg_and_rdf_references_map_like_ids() {
    let a = canon(
      r##"<d xmlns:svg="http://www.w3.org/2000/svg"><p xml:id="S1.p1"/><ref href="#S1.p1"/><rdf about="#S1.p1" aboutidref="S1.p1"/><svg:clipPath id="pgfcp1"/><svg:g clip-path="url(#pgfcp1)"/><svg:use href="#pgfpat1"/></d>"##,
    );
    let b = canon(
      r##"<d xmlns:svg="http://www.w3.org/2000/svg"><p xml:id="S1.p2"/><ref href="#S1.p2"/><rdf about="#S1.p2" aboutidref="S1.p2"/><svg:clipPath id="pgfcp2"/><svg:g clip-path="url(#pgfcp2)"/><svg:use href="#pgfpat2"/></d>"##,
    );
    let d = diff(&a, &b, 5);
    assert_eq!(
      (d.deleted, d.inserted, d.refs_retargeted),
      (0, 0, 0),
      "{d:?}"
    );
    assert_eq!(attr_value("url(#pgfcp12)"), "url(#pgfcpN)");
    assert_eq!(attr_value("#pgfpat3"), "#pgfpatN");
    // Text is not an id: `pgfplots1` keeps its number (/tmp/rev_p0r3 r4).
    assert_eq!(
      volatile("use pgfplots1 and pgfmath2"),
      "use pgfplots1 and pgfmath2"
    );
    assert_eq!(attr_value("x pgfcp12"), "x pgfcp12");
  }

  #[test]
  fn a_lost_space_is_a_changed_line() {
    let a = canon(r#"<d><p>foo <b>bar</b></p></d>"#);
    let b = canon(r#"<d><p>foo<b>bar</b></p></d>"#);
    assert_ne!(diff(&a, &b, 5).deleted, 0);
  }
}
