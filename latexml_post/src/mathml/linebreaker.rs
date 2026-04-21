//! MathML line-breaking algorithm.
//!
//! Port of `LaTeXML::Post::MathML::Linebreaker` (1053 lines of Perl).
//! Implements line-breaking for long Presentation MathML expressions
//! by finding optimal breakpoints and inserting `<mspace linebreak="newline"/>`.
//!
//! Strategy (from Perl source):
//! 1. If top-level has trailing punctuation, remove it to add back later
//! 2. Find all layouts that fit within a specified width (top-down)
//!    - Find possible breaks within current node
//!    - Recursively find possible layouts for children
//!    - Pattern of breaks depends on tag (sub/superscripts don't break)
//!    - Each combination scored by width + penalty
//! 3. Apply the best layout's breaks

use std::collections::HashSet;

use crate::document::NodeData;

/// Penalty constants (matching Perl).
const NOBREAK: i32 = 99999999;
const POORBREAK_FACTOR: i32 = 20;
const BADBREAK_FACTOR: i32 = 100;
const PENALTY_OK: i32 = 5;
const PENALTY_LIMIT: i32 = 1000;
const CONVERSION_FACTOR: i32 = 2;

/// Operators that prefer breaking BEFORE them.
fn break_before_ops() -> HashSet<&'static str> {
  ["+", "-", "\u{00B1}", "\u{2212}", "\u{2213}"]
    .into_iter()
    .collect()
}

/// Operators that prefer breaking AFTER them.
fn break_after_ops() -> HashSet<&'static str> { [","].into_iter().collect() }

/// Relation operators (good breakpoints).
fn relation_ops() -> HashSet<&'static str> {
  [
    "=", "<", ">", "\u{2264}", "\u{2265}", "\u{2260}", "\u{226A}", "\u{2261}", "\u{223C}",
    "\u{2243}", "\u{224D}", "\u{2248}", "\u{221D}",
  ]
  .into_iter()
  .collect()
}

/// Fence/delimiter operators (bad breakpoints).
fn fence_ops() -> HashSet<&'static str> {
  [
    "(", ")", "[", "]", "{", "}", "|", "||", "\u{2308}", "\u{2309}", "\u{230A}", "\u{230B}",
    "\u{27E8}", "\u{27E9}", "\u{27EA}", "\u{27EB}", "\u{27EE}", "\u{27EF}",
  ]
  .into_iter()
  .collect()
}

/// Separator operators.
fn separator_ops() -> HashSet<&'static str> { [",", ";", ".", "\u{2063}"].into_iter().collect() }

/// Invisible times → visible times conversion for breakpoints.
fn convert_ops() -> Vec<(&'static str, &'static str)> {
  vec![("\u{2062}", "\u{00D7}")] // INVISIBLE TIMES → MULTIPLICATION SIGN
}

/// A single layout option for a MathML subtree.
#[derive(Debug, Clone)]
pub struct Layout {
  /// Total width in "em-like" units.
  pub width:     f64,
  /// Total penalty score.
  pub penalty:   i32,
  /// Whether this layout contains line breaks.
  pub has_break: bool,
  /// Break positions (indices into children).
  pub breaks:    Vec<usize>,
  /// Indentation depth for continuation lines.
  pub indent:    f64,
}

impl Layout {
  fn no_break(width: f64) -> Self {
    Layout {
      width,
      penalty: 0,
      has_break: false,
      breaks: vec![],
      indent: 0.0,
    }
  }
}

/// Line-breaking configuration.
pub struct Linebreaker {
  /// Target line width in "em" units.
  pub target_width: f64,
}

impl Linebreaker {
  pub fn new(target_width: f64) -> Self { Linebreaker { target_width } }

  /// Find the best layout for a MathML expression that fits within target width.
  ///
  /// Port of `Linebreaker::bestFitToWidth`.
  pub fn best_fit_to_width(&self, node: &NodeData) -> Layout {
    let layouts = self.find_layouts(node, 0);
    // Find the best layout: widest that fits, lowest penalty
    let mut best = Layout::no_break(self.estimate_width(node));
    for layout in &layouts {
      if layout.width <= self.target_width && (!best.has_break || layout.penalty < best.penalty) {
        best = layout.clone();
      }
    }
    best
  }

  /// Recursively find possible layouts for a node.
  ///
  /// Port of `Linebreaker::findLayouts`.
  fn find_layouts(&self, node: &NodeData, depth: usize) -> Vec<Layout> {
    match node {
      NodeData::Text(s) => {
        vec![Layout::no_break(estimate_text_width(s))]
      },
      NodeData::Element { tag, children, .. } => {
        // Don't break inside scripts, fractions, roots
        if tag.starts_with("m:msub")
          || tag.starts_with("m:msup")
          || tag == "m:mfrac"
          || tag == "m:msqrt"
          || tag == "m:mroot"
          || tag == "m:munder"
          || tag == "m:mover"
          || tag == "m:munderover"
        {
          let w: f64 = children.iter().map(|c| self.estimate_width(c)).sum();
          return vec![Layout::no_break(w)];
        }

        // For mrow and similar containers, find breakpoints
        if tag == "m:mrow" || tag == "m:math" {
          return self.find_mrow_layouts(children, depth);
        }

        // Default: sum of children, no breaks
        let w: f64 = children.iter().map(|c| self.estimate_width(c)).sum();
        vec![Layout::no_break(w)]
      },
      NodeData::XmlNode(_) => vec![Layout::no_break(1.0)],
    }
  }

  /// Find breakpoint layouts for an mrow's children.
  fn find_mrow_layouts(&self, children: &[NodeData], _depth: usize) -> Vec<Layout> {
    let total_width: f64 = children.iter().map(|c| self.estimate_width(c)).sum();

    // No break needed
    if total_width <= self.target_width {
      return vec![Layout::no_break(total_width)];
    }

    let break_before = break_before_ops();
    let break_after = break_after_ops();
    let relation = relation_ops();

    // Find potential breakpoints
    let mut layouts = vec![Layout::no_break(total_width)];

    for (i, child) in children.iter().enumerate() {
      if let NodeData::Element { tag, children: inner, .. } = child {
        if tag == "m:mo" {
          if let Some(NodeData::Text(text)) = inner.first() {
            let penalty = if relation.contains(text.as_str()) {
              PENALTY_OK
            } else if break_before.contains(text.as_str()) {
              PENALTY_OK * POORBREAK_FACTOR
            } else if break_after.contains(text.as_str()) {
              PENALTY_OK * 2
            } else {
              PENALTY_OK * BADBREAK_FACTOR
            };

            // Create a layout with a break at this point
            let indent = 2.0; // em
            let width_after = children[i + 1..]
              .iter()
              .map(|c| self.estimate_width(c))
              .sum::<f64>()
              + indent;
            let width_before: f64 = children[..i].iter().map(|c| self.estimate_width(c)).sum();
            let max_line = width_before.max(width_after);

            layouts.push(Layout {
              width: max_line,
              penalty,
              has_break: true,
              breaks: vec![i],
              indent,
            });
          }
        }
      }
    }

    // Sort by width, then penalty; prune dominated layouts
    layouts.sort_by(|a, b| {
      a.width
        .partial_cmp(&b.width)
        .unwrap()
        .then(a.penalty.cmp(&b.penalty))
    });

    // Prune: remove layouts wider than target with higher penalty than narrower ones
    let mut pruned = Vec::new();
    let mut best_penalty = i32::MAX;
    for layout in layouts {
      if layout.penalty < best_penalty || layout.width <= self.target_width {
        best_penalty = best_penalty.min(layout.penalty);
        pruned.push(layout);
      }
    }

    pruned
  }

  /// Estimate the width of a node in "em-like" units.
  fn estimate_width(&self, node: &NodeData) -> f64 {
    match node {
      NodeData::Text(s) => estimate_text_width(s),
      NodeData::Element { children, .. } => children
        .iter()
        .map(|c| self.estimate_width(c))
        .sum::<f64>()
        .max(0.5),
      NodeData::XmlNode(_) => 1.0,
    }
  }

  /// Apply a layout's breaks to a MathML expression.
  ///
  /// Port of `Linebreaker::applyLayout`.
  pub fn apply_layout(&self, node: &NodeData, layout: &Layout) -> NodeData {
    if !layout.has_break {
      return node.clone();
    }
    // Insert mspace linebreak="newline" at each break position
    match node {
      NodeData::Element { tag, attributes, children } => {
        let mut new_children = Vec::new();
        for (i, child) in children.iter().enumerate() {
          new_children.push(child.clone());
          if layout.breaks.contains(&i) {
            new_children.push(NodeData::Element {
              tag:        "m:mspace".to_string(),
              attributes: Some(std::collections::HashMap::from([(
                "linebreak".to_string(),
                "newline".to_string(),
              )])),
              children:   vec![],
            });
          }
        }
        NodeData::Element {
          tag:        tag.clone(),
          attributes: attributes.clone(),
          children:   new_children,
        }
      },
      _ => node.clone(),
    }
  }
}

/// Estimate text width in em-like units (rough approximation).
fn estimate_text_width(s: &str) -> f64 { s.chars().count() as f64 * 0.6 }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn penalty_constants_ordering() {
    // NOBREAK must dwarf every other penalty (used as "never break here" sentinel).
    assert!(NOBREAK > PENALTY_LIMIT);
    assert!(PENALTY_LIMIT > BADBREAK_FACTOR);
    assert!(BADBREAK_FACTOR > POORBREAK_FACTOR);
    assert!(POORBREAK_FACTOR > PENALTY_OK);
    assert!(PENALTY_OK > 0);
  }

  #[test]
  fn break_before_ops_contains_plus_minus() {
    let ops = break_before_ops();
    assert!(ops.contains("+"));
    assert!(ops.contains("-"));
    assert!(ops.contains("\u{00B1}")); // ±
    assert!(ops.contains("\u{2212}")); // −
  }

  #[test]
  fn break_after_ops_contains_comma() {
    let ops = break_after_ops();
    assert!(ops.contains(","));
  }

  #[test]
  fn relation_ops_contains_common() {
    let ops = relation_ops();
    assert!(ops.contains("="));
    assert!(ops.contains("<"));
    assert!(ops.contains(">"));
    assert!(ops.contains("\u{2264}")); // ≤
    assert!(ops.contains("\u{2265}")); // ≥
    assert!(ops.contains("\u{2260}")); // ≠
  }

  #[test]
  fn fence_ops_contains_parens_brackets_braces() {
    let ops = fence_ops();
    for c in ["(", ")", "[", "]", "{", "}"] {
      assert!(ops.contains(c), "missing {c}");
    }
  }

  #[test]
  fn separator_ops_distinct_from_relation() {
    let sep = separator_ops();
    let rel = relation_ops();
    // Separators and relations should not overlap.
    for s in &sep {
      assert!(
        !rel.contains(s),
        "{s:?} should not be both separator and relation"
      );
    }
    // Common separators present.
    assert!(sep.contains(","));
    assert!(sep.contains(";"));
  }

  #[test]
  fn convert_ops_invisible_to_visible_times() {
    let pairs = convert_ops();
    assert_eq!(pairs.len(), 1);
    assert_eq!(
      pairs[0],
      ("\u{2062}", "\u{00D7}"),
      "INVISIBLE TIMES → MULTIPLICATION SIGN"
    );
  }

  #[test]
  fn layout_no_break_has_zero_penalty() {
    let l = Layout::no_break(5.0);
    assert_eq!(l.width, 5.0);
    assert_eq!(l.penalty, 0);
    assert!(!l.has_break);
    assert!(l.breaks.is_empty());
    assert_eq!(l.indent, 0.0);
  }

  #[test]
  fn estimate_text_width_proportional_to_length() {
    // 0.6 em per character (rough).
    assert!((estimate_text_width("") - 0.0).abs() < 1e-6);
    assert!((estimate_text_width("a") - 0.6).abs() < 1e-6);
    assert!((estimate_text_width("abcde") - 3.0).abs() < 1e-6);
  }

  #[test]
  fn estimate_text_width_counts_chars_not_bytes() {
    // Unicode chars count as 1 each, even multi-byte.
    // "αβγ" is 3 chars = 1.8 em, not 6 bytes.
    assert!((estimate_text_width("αβγ") - 1.8).abs() < 1e-6);
  }
}
